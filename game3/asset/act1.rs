//! Act 1, as bytes: the story rides in a custom section, nothing else.

#[used]
#[link_section = "se.asset"]
static DATA: [u8; include_bytes!("act1/story.rpy").len()] = *include_bytes!("act1/story.rpy");
