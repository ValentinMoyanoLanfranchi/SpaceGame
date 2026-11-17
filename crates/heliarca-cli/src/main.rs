//! Ejecutor sin interfaz de Heliarca.

/// Texto que imprime el ejecutable al arrancar.
fn banner() -> String {
    format!("heliarca-cli (sim {})", heliarca_sim::version())
}

fn main() {
    println!("{}", banner());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_banner_incluye_la_version_de_sim() {
        assert!(banner().contains(heliarca_sim::version()));
    }
}
