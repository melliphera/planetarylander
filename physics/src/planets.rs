use agc_utils::{SolarFp, SolarVec3D, StepVec3D};
use arrayvec::ArrayVec;
use fixedstr::str16;

/// stored gravity as a fixed point and a bit scalar. For the Sun, the scalar is 20, for gas giants its 10. For all else it is 0. 
#[derive(Debug, Clone, Copy)]
pub struct Gravity {
    pub stored_solar: SolarFp, 
    pub scale: u8 
}

impl Gravity {
    pub fn to_f64(self) -> f64 {
        //! produces the float represented by this value; including correctly applying the scale.
        self.stored_solar.to_f64() * (2.0f64).powi(self.scale as i32)
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub name: str16,
    pub gravity: Gravity, // G*m_1; divide by d^2 for acceleration of external body. Bitshifted by grav_scale
    pub position: SolarVec3D,
    pub velocity: StepVec3D,
    pub parent_id: Option<usize>,
    pub orbit_influencers: ArrayVec<usize, 20>,
    pub id: usize,
}

impl Body {
    const fn new(
        name: &str,
        gravity: f64,
        scale: u8,
        position: SolarVec3D,
        velocity: StepVec3D,
        parent_id: usize,
        id: usize,
    ) -> Self {
        // valid function for any Body with a parent - all but Sol
        Body {
            name: str16::const_make(name),
            gravity: Gravity {
                stored_solar: SolarFp::from_f64_trusted(gravity),
                scale
            },
            position,
            velocity,
            parent_id: Some(parent_id),
            orbit_influencers: ArrayVec::new_const(),
            id,
        }
    }

    pub fn fill_influencers(&mut self, body_list: &[Body; 10]) {
        //! each body is influenced by their parent, siblings and children

        for (i, body) in body_list.iter().enumerate() {
            // each body is influenced by their parent, sibling and children:
            if Some(i) == self.parent_id            // parent,
                || body.parent_id == self.parent_id // siblings
                || body.parent_id == Some(self.id)
            // children
            {
                self.orbit_influencers.push(i)
            }
        }
    }
}

pub const N_BODIES: usize = 10;
pub const BODIES: [Body; N_BODIES] = [
    // ESTABLISHING Sun Centre at Epoch (SCE) as a static reference frame for the entire simulation.
    // Epoch used for this and all other initial data is Jan-1-2000 00:00.
    Body {
        name: str16::const_make("Sol"),
        gravity: Gravity {
            stored_solar: SolarFp::from_f64_trusted(1.26558e14),
            scale: 20,
        },
        position: SolarVec3D::from_floats_trusted(0.0, 0.0, 0.0),
        velocity: StepVec3D::from_floats_trusted(0.0, 0.0, 0.0),
        parent_id: None,
        orbit_influencers: ArrayVec::new_const(),
        id: 0,
    },
    Body::new(
        "Mercury",
        2.20375e13,
        0,
        SolarVec3D::from_floats_trusted(
            -2.105_262_107_244_07E10,
            -6.640_663_812_253_43E10,
            -3.492_445_946_577_72E9,
        ),
        StepVec3D::from_floats_trusted(
            3.665298704187096E+04,
            -1.228983806940175E+04,
            -4.368_173_036_243_59E3,
        ),
        0,
        1,
    ),
    Body::new(
        "Venus",
        3.24924e14,
        0,
        SolarVec3D::from_floats_trusted(
            -1.075_055_502_719_85E11,
            -3.366520666522362E+09,
            6.159219789239045E+09,
        ),
        StepVec3D::from_floats_trusted(
            8.891597859686224E+02,
            -3.515920774137907E+04,
            -5.318594228644749E+02,
        ),
        0,
        2,
    ),
    Body::new(
        "Earth",
        3.98438e14,
        0,
        SolarVec3D::from_floats_trusted(
            -2.521092855899356E+10,
            1.449279195838006E+11,
            -6.164165719002485E+05,
        ),
        StepVec3D::from_floats_trusted(
            -2.983983333677879E+04,
            -5.207633902410673E+03,
            6.168441184239981E-02,
        ),
        0,
        3,
    ),
    Body::new(
        "Mars",
        4.27277e13,
        0,
        SolarVec3D::from_floats_trusted(
            2.079950549836171E+11,
            -3.143009713942494E+09,
            -5.178781243488781E+09,
        ),
        StepVec3D::from_floats_trusted(
            1.295003552976085E+03,
            2.629442066947034E+04,
            5.190097459225722E+02,
        ),
        0,
        4,
    ),
    Body::new(
        "Jupiter",
        1.23183e14,
        10,
        SolarVec3D::from_floats_trusted(
            5.989091645401344E+11,
            4.391225866604841E+11,
            -1.523251063025475E+10,
        ),
        StepVec3D::from_floats_trusted(
            -7.901937516136118E+03,
            1.116317703172796E+04,
            1.306_732_148_714_28E2,
        ),
        0,
        5,
    ),
    Body::new(
        "Saturn",
        3.7041e13,
        10,
        SolarVec3D::from_floats_trusted(
            9.587063371733198E+11,
            9.825652104588115E+11,
            -5.522065631225652E+10,
        ),
        StepVec3D::from_floats_trusted(
            -7.428885680409909E+03,
            6.738814240733793E+03,
            1.776643606866641E+02,
        ),
        0,
        6,
    ),
    Body::new(
        "Uranus",
        5.65811e12,
        10,
        SolarVec3D::from_floats_trusted(
            2.158774481135687E+12,
            -2.054825439980978E+12,
            -3.562364902696741E+10,
        ),
        StepVec3D::from_floats_trusted(
            4.637647623549194E+03,
            4.627191832186803E+03,
            -4.285552181289254E+01,
        ),
        0,
        7,
    ),
    Body::new(
        "Neptune",
        6.67459e12,
        10,
        SolarVec3D::from_floats_trusted(
            2.514853560731005E+12,
            -3.738847414418683E+12,
            1.903940653525472E+10,
        ),
        StepVec3D::from_floats_trusted(
            4.465802635365747E+03,
            3.075682319816144E+03,
            -1.665662221033826E+02,
        ),
        0,
        8,
    ),
    Body::new(
        "Pluto",
        8.72292e11,
        0,
        SolarVec3D::from_floats_trusted(
            -1.477558207142231E+12,
            -4.182460280867265E+12,
            8.752693291814536E+11,
        ),
        StepVec3D::from_floats_trusted(
            5.261903258542794E+03,
            -2.648936864051314E+03,
            -1.241856559011054E+03,
        ),
        0,
        9,
    ),
];
