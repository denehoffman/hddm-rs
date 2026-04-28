use hddm::Particle;

mod hddm_mc {
    include!(concat!(env!("OUT_DIR"), "/hddm_mc.rs"));
}
mod hddm_s {
    include!(concat!(env!("OUT_DIR"), "/hddm_s.rs"));
}

fn demo_event() -> hddm_mc::Hddm {
    hddm_mc::Hddm {
        physics_event: vec![hddm_mc::PhysicsEvent {
            event_no: 1,
            run_no: 90001,
            reaction: vec![hddm_mc::Reaction {
                type_: 0,
                weight: 1.0,

                beam: Some(hddm_mc::Beam {
                    type_: Particle::Gamma,
                    momentum: hddm_mc::Momentum {
                        e: 9.0,
                        px: 0.0,
                        py: 0.0,
                        pz: 9.0,
                        momentum_double: None,
                    },
                    properties: hddm_mc::Properties {
                        charge: 0,
                        mass: 0.0,
                    },
                }),

                target: Some(hddm_mc::Target {
                    type_: Particle::Proton,
                    momentum: hddm_mc::Momentum {
                        e: 0.938272,
                        px: 0.0,
                        py: 0.0,
                        pz: 0.0,
                        momentum_double: None,
                    },
                    properties: hddm_mc::Properties {
                        charge: 1,
                        mass: 0.938272,
                    },
                }),

                vertex: vec![hddm_mc::Vertex {
                    origin: hddm_mc::Origin {
                        t: 0.0,
                        vx: 0.0,
                        vy: 0.0,
                        vz: 65.0,
                    },
                    product: vec![hddm_mc::Product {
                        id: 1,
                        parentid: 0,
                        pdgtype: 211,
                        type_: Particle::PiPlus,
                        decay_vertex: 0,
                        mech: 0,
                        momentum: hddm_mc::Momentum {
                            e: 1.0,
                            px: 0.1,
                            py: 0.0,
                            pz: 0.9,
                            momentum_double: None,
                        },
                        properties: Some(hddm_mc::Properties {
                            charge: 1,
                            mass: 0.13957,
                        }),
                    }],
                }],

                random: Some(hddm_mc::Random {
                    seed1: 1,
                    seed2: 2,
                    seed3: 3,
                    seed4: 4,
                }),
            }],
        }],
    }
}

fn write_read(path: &str, compression: hddm::Compression) -> hddm::HddmResult<()> {
    let event = demo_event();

    let mut out = hddm_mc::create_with_compression(path, compression)?;

    out.write_record(&event)?;
    out.finish()?;

    let mut input = hddm_mc::open(path)?;
    let decoded = input.read_record::<hddm_mc::Hddm>()?;
    assert_eq!(decoded.as_ref(), Some(&event));

    println!("wrote {path}");
    Ok(())
}

fn main() -> hddm::HddmResult<()> {
    write_read("/tmp/demo-sample-mc-none.hddm", hddm::Compression::None)?;
    write_read("/tmp/demo-sample-mc-zlib.hddm", hddm::Compression::Zlib)?;
    write_read("/tmp/demo-sample-mc-bzip2.hddm", hddm::Compression::Bzip2)?;
    Ok(())
}
