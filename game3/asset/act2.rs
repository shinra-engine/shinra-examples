//! Act 2, as bytes: the story rides in a custom section, nothing else.

#[used]
#[link_section = "se.asset"]
static DATA: [u8; include_bytes!("act2/story.rpy").len()] = *include_bytes!("act2/story.rpy");
