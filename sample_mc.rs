#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
pub const MODEL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<HDDM class=\"mc\" version=\"1.0\" xmlns=\"http://www.gluex.org/hddm\">\n  <physicsEvent eventNo=\"int\" maxOccurs=\"unbounded\" runNo=\"int\">\n    <reaction maxOccurs=\"unbounded\" minOccurs=\"0\" type=\"int\" weight=\"float\">\n      <beam minOccurs=\"0\" type=\"Particle_t\">\n        <momentum E=\"float\" px=\"float\" py=\"float\" pz=\"float\">\n          <momentum_double minOccurs=\"0\" E=\"double\" px=\"double\" py=\"double\" pz=\"double\" />\n        </momentum>\n        <properties charge=\"int\" mass=\"float\" />\n      </beam>\n      <target minOccurs=\"0\" type=\"Particle_t\">\n        <momentum E=\"float\" px=\"float\" py=\"float\" pz=\"float\">\n          <momentum_double minOccurs=\"0\" E=\"double\" px=\"double\" py=\"double\" pz=\"double\" />\n        </momentum>\n        <properties charge=\"int\" mass=\"float\" />\n      </target>\n      <vertex maxOccurs=\"unbounded\">\n        <product decayVertex=\"int\" id=\"int\" maxOccurs=\"unbounded\" mech=\"int\" parentid=\"int\" pdgtype=\"int\" type=\"Particle_t\">\n          <momentum E=\"float\" px=\"float\" py=\"float\" pz=\"float\">\n            <momentum_double minOccurs=\"0\" E=\"double\" px=\"double\" py=\"double\" pz=\"double\" />\n          </momentum>\n          <properties charge=\"int\" mass=\"float\" minOccurs=\"0\" />\n        </product>\n        <origin t=\"float\" vx=\"float\" vy=\"float\" vz=\"float\" />\n      </vertex>\n      <random maxOccurs=\"1\" minOccurs=\"0\" seed1=\"int\" seed2=\"int\" seed3=\"int\" seed4=\"int\" />\n    </reaction>\n  </physicsEvent>\n</HDDM>\n";
pub const HDDM_CLASS: &str = "mc";
pub type Root = Hddm;
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct MomentumDouble {
    pub e: f64,
    pub px: f64,
    pub py: f64,
    pub pz: f64,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Momentum {
    pub e: f32,
    pub px: f32,
    pub py: f32,
    pub pz: f32,
    pub momentum_double: Option<MomentumDouble>,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Properties {
    pub charge: i32,
    pub mass: f32,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Beam {
    pub particle_type: ::gluex_core::Particle,
    pub momentum: Option<Momentum>,
    pub properties: Option<Properties>,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Target {
    pub particle_type: ::gluex_core::Particle,
    pub momentum: Option<Momentum>,
    pub properties: Option<Properties>,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Product {
    pub decay_vertex: i32,
    pub id: i32,
    pub mech: i32,
    pub parentid: i32,
    pub pdgtype: i32,
    pub particle_type: ::gluex_core::Particle,
    pub momentum: Option<Momentum>,
    pub properties: Option<Properties>,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Origin {
    pub t: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Vertex {
    pub product: Vec<Product>,
    pub origin: Option<Origin>,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Random {
    pub seed1: i32,
    pub seed2: i32,
    pub seed3: i32,
    pub seed4: i32,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Reaction {
    pub particle_type: i32,
    pub weight: f32,
    pub beam: Option<Beam>,
    pub target: Option<Target>,
    pub vertex: Vec<Vertex>,
    pub random: Option<Random>,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct PhysicsEvent {
    pub event_no: i32,
    pub run_no: i32,
    pub reaction: Vec<Reaction>,
}
#[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
pub struct Hddm {
    pub physics_event: Vec<PhysicsEvent>,
}
impl Hddm {
    pub fn writer<P: AsRef<std::path::Path>>(
        path: P,
    ) -> ::hddm::HddmResult<::hddm::HddmFileWriter> {
        ::hddm::HddmFileWriter::create(path, MODEL)
    }
}
