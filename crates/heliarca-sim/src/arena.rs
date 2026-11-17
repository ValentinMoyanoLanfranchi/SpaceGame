//! Arena con generación: almacén indexado de entidades con ids que no se reciclan a ciegas
//! (doc 21 §2 «Identificadores y arenas»).
//!
//! # Cómo se concilia `u32` con la generación
//!
//! El id tipado sigue siendo un único `u32` ([`EntityId`]), pero la [`Arena`] lo **empaqueta**:
//! los 24 bits bajos son el índice del slot ([`MAX_SLOTS`] = 16 777 216 slots) y los 8 bits altos
//! la generación (0..=[`MAX_GENERATION`]). Al liberar un slot su generación sube en uno, así que
//! un id viejo ya no coincide con el slot reutilizado y [`Arena::get`] devuelve `None`.
//!
//! # Políticas explícitas (sin panics ni desbordes silenciosos)
//!
//! - **Generación agotada:** si un slot llega a [`MAX_GENERATION`] y se libera, queda *retirado*:
//!   no vuelve a la lista de libres y nunca se reutiliza (no se envuelve a 0, porque eso podría
//!   resucitar ids viejos). Cuesta un slot vacío por cada 256 usos de ese slot.
//! - **Arena llena:** [`Arena::insert`] devuelve [`ArenaFull`] en vez de entrar en pánico.
//! - **Orden estable:** la iteración recorre los slots por índice ascendente; no depende de
//!   hashes ni de la historia salvo por qué slot ocupa cada elemento. La reutilización de slots
//!   es determinista: se reusa primero el último liberado (pila LIFO).
//!
//! Es un tipo **interno** del crate (no forma parte de la API pública); el ejemplo de reutilización
//! de slots y generación está en la prueba `ejemplo_de_reutilizacion_de_slots`.

use std::fmt;
use std::marker::PhantomData;

use crate::ids::EntityId;

const INDEX_BITS: u32 = 24;
const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;

/// Cantidad máxima de slots de una arena (2^24).
pub const MAX_SLOTS: usize = 1 << INDEX_BITS;
/// Mayor generación de un slot; al liberarlo en ese valor el slot se retira.
pub const MAX_GENERATION: u8 = u8::MAX;

/// La arena no admite más slots ([`MAX_SLOTS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaFull;

impl fmt::Display for ArenaFull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "la arena está llena ({MAX_SLOTS} slots)")
    }
}

impl std::error::Error for ArenaFull {}

#[derive(Debug, Clone)]
struct Slot<T> {
    generation: u8,
    value: Option<T>,
}

/// Arena de valores `T` direccionados por ids tipados `I` con generación.
#[derive(Debug, Clone)]
pub struct Arena<I: EntityId, T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    len: usize,
    _id: PhantomData<fn() -> I>,
}

impl<I: EntityId, T> Default for Arena<I, T> {
    fn default() -> Self {
        Self::new()
    }
}

fn pack(index: u32, generation: u8) -> u32 {
    (u32::from(generation) << INDEX_BITS) | (index & INDEX_MASK)
}

fn unpack(raw: u32) -> (usize, u8) {
    let generation = u8::try_from(raw >> INDEX_BITS).unwrap_or(u8::MAX);
    (
        usize::try_from(raw & INDEX_MASK).unwrap_or(usize::MAX),
        generation,
    )
}

impl<I: EntityId, T> Arena<I, T> {
    /// Crea una arena vacía.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            len: 0,
            _id: PhantomData,
        }
    }

    /// Cantidad de elementos vivos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Indica si no hay elementos vivos.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Inserta `value` y devuelve su id.
    ///
    /// # Errors
    ///
    /// [`ArenaFull`] si se alcanzó [`MAX_SLOTS`] y no hay slots libres.
    pub fn insert(&mut self, value: T) -> Result<I, ArenaFull> {
        if let Some(index) = self.free.pop() {
            let slot = self.slots.get_mut(index as usize).ok_or(ArenaFull)?;
            slot.value = Some(value);
            self.len += 1;
            return Ok(I::from_raw(pack(index, slot.generation)));
        }
        if self.slots.len() >= MAX_SLOTS {
            return Err(ArenaFull);
        }
        let index = u32::try_from(self.slots.len()).map_err(|_| ArenaFull)?;
        self.slots.push(Slot {
            generation: 0,
            value: Some(value),
        });
        self.len += 1;
        Ok(I::from_raw(pack(index, 0)))
    }

    fn slot(&self, id: I) -> Option<&Slot<T>> {
        let (index, generation) = unpack(id.raw());
        self.slots.get(index).filter(|s| s.generation == generation)
    }

    /// Elemento de `id`, o `None` si fue liberado (aunque su slot se haya reutilizado).
    #[must_use]
    pub fn get(&self, id: I) -> Option<&T> {
        self.slot(id)?.value.as_ref()
    }

    /// Versión mutable de [`Arena::get`].
    #[must_use]
    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        let (index, generation) = unpack(id.raw());
        self.slots
            .get_mut(index)
            .filter(|s| s.generation == generation)?
            .value
            .as_mut()
    }

    /// Indica si `id` resuelve a un elemento vivo.
    #[must_use]
    pub fn contains(&self, id: I) -> bool {
        self.get(id).is_some()
    }

    /// Libera `id` y devuelve su valor; `None` si no resolvía a un elemento vivo.
    ///
    /// Sube la generación del slot; si ya estaba en [`MAX_GENERATION`] el slot se retira.
    pub fn remove(&mut self, id: I) -> Option<T> {
        let (index, generation) = unpack(id.raw());
        let slot = self
            .slots
            .get_mut(index)
            .filter(|s| s.generation == generation)?;
        let value = slot.value.take()?;
        self.len -= 1;
        if let Some(next) = slot.generation.checked_add(1) {
            slot.generation = next;
            // `index` salió de `unpack` de un id válido: cabe en 24 bits.
            self.free.push(u32::try_from(index).unwrap_or(u32::MAX));
        }
        Some(value)
    }

    /// Recorre los elementos vivos como `(id, &valor)` por índice de slot ascendente.
    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> + '_ {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            let value = slot.value.as_ref()?;
            let index = u32::try_from(index).ok()?;
            Some((I::from_raw(pack(index, slot.generation)), value))
        })
    }

    /// Como [`Arena::iter`] pero con acceso mutable a los valores.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (I, &mut T)> + '_ {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| {
                let generation = slot.generation;
                let value = slot.value.as_mut()?;
                let index = u32::try_from(index).ok()?;
                Some((I::from_raw(pack(index, generation)), value))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{BodyId, ShipId};

    type Ships = Arena<ShipId, &'static str>;

    #[test]
    fn ejemplo_de_reutilizacion_de_slots() {
        let mut naves: Arena<ShipId, &str> = Arena::new();
        let a = naves.insert("alba").unwrap();
        naves.remove(a);
        let b = naves.insert("boreal").unwrap(); // reutiliza el slot
        assert_eq!(naves.get(a), None); // el id viejo no resuelve al objeto nuevo
        assert_eq!(naves.get(b), Some(&"boreal"));
    }

    #[test]
    fn insertar_obtener_y_contar() {
        let mut a = Ships::new();
        assert!(a.is_empty());
        let x = a.insert("x").unwrap();
        let y = a.insert("y").unwrap();
        assert_eq!(a.len(), 2);
        assert_eq!(a.get(x), Some(&"x"));
        *a.get_mut(y).unwrap() = "y2";
        assert_eq!(a.get(y), Some(&"y2"));
        assert!(a.contains(x));
    }

    #[test]
    fn id_reutilizado_no_resuelve_al_objeto_viejo() {
        let mut a = Ships::new();
        let viejo = a.insert("viejo").unwrap();
        assert_eq!(a.remove(viejo), Some("viejo"));
        assert_eq!(a.get(viejo), None);
        let nuevo = a.insert("nuevo").unwrap();
        // Mismo slot, distinta generación.
        assert_ne!(viejo, nuevo);
        assert_eq!(a.get(viejo), None);
        assert_eq!(a.get_mut(viejo), None);
        assert_eq!(a.remove(viejo), None);
        assert_eq!(a.get(nuevo), Some(&"nuevo"));
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn doble_remove_es_none() {
        let mut a = Ships::new();
        let x = a.insert("x").unwrap();
        assert_eq!(a.remove(x), Some("x"));
        assert_eq!(a.remove(x), None);
        assert_eq!(a.len(), 0);
    }

    #[test]
    fn iteracion_en_orden_de_indice_tras_borrar_y_reinsertar() {
        let mut a = Arena::<BodyId, u32>::new();
        let ids: Vec<_> = (0..5).map(|n| a.insert(n).unwrap()).collect();
        a.remove(ids[1]);
        a.remove(ids[3]);
        // Reusa primero el último liberado (slot 3), luego el 1.
        a.insert(30).unwrap();
        a.insert(10).unwrap();
        let valores: Vec<u32> = a.iter().map(|(_, v)| *v).collect();
        assert_eq!(valores, [0, 10, 2, 30, 4]);
        let indices: Vec<usize> = a.iter().map(|(id, _)| unpack(id.raw()).0).collect();
        assert_eq!(indices, [0, 1, 2, 3, 4]);
        for (_, v) in a.iter_mut() {
            *v += 1;
        }
        let valores: Vec<u32> = a.iter().map(|(_, v)| *v).collect();
        assert_eq!(valores, [1, 11, 3, 31, 5]);
    }

    #[test]
    fn iter_devuelve_ids_que_resuelven() {
        let mut a = Ships::new();
        a.insert("a").unwrap();
        let b = a.insert("b").unwrap();
        a.remove(b);
        a.insert("c").unwrap();
        for (id, v) in a.iter() {
            assert_eq!(a.get(id), Some(v));
        }
    }

    #[test]
    fn generacion_agotada_retira_el_slot() {
        let mut a = Ships::new();
        let mut id = a.insert("x").unwrap();
        // 255 liberaciones suben la generación hasta MAX_GENERATION; todas reusan el slot 0.
        for _ in 0..MAX_GENERATION {
            a.remove(id);
            id = a.insert("x").unwrap();
            assert_eq!(unpack(id.raw()).0, 0);
        }
        assert_eq!(unpack(id.raw()).1, MAX_GENERATION);
        // Liberar en la generación máxima retira el slot: el próximo insert usa otro.
        a.remove(id);
        let siguiente = a.insert("y").unwrap();
        assert_eq!(unpack(siguiente.raw()).0, 1);
        assert_eq!(a.get(id), None);
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn id_inventado_no_resuelve() {
        let a = Ships::new();
        assert_eq!(a.get(ShipId::from_raw(u32::MAX)), None);
        assert_eq!(a.get(ShipId::from_raw(0)), None);
    }

    #[test]
    fn error_de_arena_llena_tiene_mensaje() {
        assert_ne!(ArenaFull.to_string(), "");
    }
}
