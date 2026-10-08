//! Asset paths in one place.
//!
//! `F1_STAIRS` is only ever a *variant* picture (the carousel target of both
//! halls), never a lookup key. `ELEVATOR` is both: it is the beat's one shot
//! *and* the room's lookup key, which is why the fall has to open on a picture
//! rather than on a frame list.

pub const F1_STREET_1: &str = "tex/rooms/floor_1/street_to_home_1.png";
pub const F1_STREET_2: &str = "tex/rooms/floor_1/street_to_home_2.png";
pub const F1_CONCIERGE: &str = "tex/rooms/floor_1/room_concierge.png";
pub const F1_CONCIERGE_DARK: &str = "tex/rooms/floor_1/room_concierge_dark.png";
pub const F1_HALL: &str = "tex/rooms/floor_1/room_with_elevator_floor_1.png";
pub const F1_HALL_DEAD: &str = "tex/rooms/floor_1/room_with_elevator_floor_1_dont_work.png";
pub const F1_STAIRS: &str = "tex/rooms/floor_1/stairs_1_floor.png";
/// The lift's interior, and the first of the nineteen frames it falls through.
///
/// A frame rather than a room of its own: the room is looked up by the picture it
/// opens on, and the animation's own list starts here anyway, so naming the
/// folder once in `act_one` beats naming frame one in two places.
pub const ELEVATOR: &str = "tex/rooms/elevator_inside/1.png";
pub const B_HALL: &str = "tex/rooms/basement/basement_with_elevator.png";
pub const B_CORRIDOR: &str = "tex/rooms/basement/basement_stairs.png";
pub const B_DEEP: &str = "tex/rooms/basement/basement_stairs_center_room.png";
pub const B_EXIT: &str = "tex/rooms/basement/basement_stairs_left_room.png";
pub const B_STREET_1: &str = "tex/rooms/basement/stairs_to_street_1.png";
pub const B_STREET_2: &str = "tex/rooms/basement/stairs_to_street_2.png";
pub const STAIRS_1: &str = "tex/rooms/stairs/stairs_1.png";
pub const STAIRS_2: &str = "tex/rooms/stairs/stairs_2.png";
pub const F2_HALL: &str = "tex/rooms/floor_2/room_1.png";
pub const F2_CORRIDOR: &str = "tex/rooms/floor_2/room_2.png";
pub const AP_1: &str = "tex/rooms/floor_2/ap_1.png";
pub const AP_2: &str = "tex/rooms/floor_2/ap_2.png";
pub const AP_3: &str = "tex/rooms/floor_2/ap_3.png";
pub const MY_FLOOR: &str = "tex/rooms/my_floor/room_with_elevator_floor_my.png";

// The two faces of the carousel control, named for where each one goes rather than
// for what it looks like: each is a picture of the destination with a chevron
// pointing at it, so the constant says what pressing it does.
pub const CAROUSEL_TO_STAIRS: &str = "tex/ui/carusel_tex/to_stairs.png";
pub const CAROUSEL_TO_ELEVATOR: &str = "tex/ui/carusel_tex/to_elevator.png";
