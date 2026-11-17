//! Registro de comandos y reproducción desde el historial, sólo con la API pública.

use heliarca_sim::{
    CommandBuildError, CommandOutcome, CommandRegistry, EventKind, Increment, Payload, Scenario,
    Sim, Value,
};

fn mark(label: &str) -> Payload {
    Payload::new().with("label", Value::Text(label.into()))
}

#[test]
fn debug_mark_se_aplica_al_comienzo_del_siguiente_paso_y_deja_un_evento() {
    let registry = CommandRegistry::builtin();
    let mut sim = Sim::new(Scenario::default(), 1);
    sim.submit_command(registry.build("debug.mark", &mark("a")).unwrap())
        .unwrap();
    assert_eq!(sim.events().len(), 0, "enviar no aplica");
    sim.advance(Increment::D1).unwrap();
    let events: Vec<_> = sim.events().iter().collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, EventKind::DebugMark);
    assert_eq!(
        events[0].payload.get("label"),
        Some(&Value::Text("a".into()))
    );
    assert_eq!(sim.command_history()[0].outcome, CommandOutcome::Applied);
}

#[test]
fn el_historial_reconstruido_con_el_registro_reproduce_el_hash() {
    let registry = CommandRegistry::builtin();
    let increments = [Increment::D1, Increment::H1, Increment::D5, Increment::S5];
    let mut live = Sim::new(Scenario::default(), 7);
    for (step, increment) in increments.into_iter().enumerate() {
        // Dos comandos en el primer paso y uno en cada paso par: distintos pasos y órdenes.
        if step == 0 || step % 2 == 0 {
            let command = registry
                .build("debug.mark", &mark(&format!("paso {step}")))
                .unwrap();
            live.submit_command(command).unwrap();
        }
        if step == 0 {
            let command = registry.build("debug.mark", &mark("segundo")).unwrap();
            live.submit_command(command).unwrap();
        }
        live.advance(increment).unwrap();
    }
    assert_eq!(live.command_history().len(), 3);

    // Reproducción: mismo seed, y cada registro (kind + args) reconstruido y reenviado antes de
    // su paso.
    let mut replay = Sim::new(Scenario::default(), live.seed());
    for (step, increment) in increments.into_iter().enumerate() {
        for record in live
            .command_history()
            .iter()
            .filter(|r| r.step == step as u64)
        {
            let command = registry.build(record.kind, &record.args).unwrap();
            replay.submit_command(command).unwrap();
        }
        replay.advance(increment).unwrap();
    }
    assert_eq!(replay.state_hash(), live.state_hash());
    assert_eq!(replay.command_history(), live.command_history());

    // Y un historial distinto da otro hash.
    let mut other = Sim::new(Scenario::default(), live.seed());
    for increment in increments {
        other.advance(increment).unwrap();
    }
    assert_ne!(other.state_hash(), live.state_hash());
}

#[test]
fn los_errores_de_construccion_son_tipados() {
    let registry = CommandRegistry::builtin();
    assert!(matches!(
        registry.build("flota.crear", &Payload::new()),
        Err(CommandBuildError::UnknownKind { .. })
    ));
    assert!(matches!(
        registry.build("debug.mark", &Payload::new()),
        Err(CommandBuildError::MissingArgument { argument: "label" })
    ));
}
