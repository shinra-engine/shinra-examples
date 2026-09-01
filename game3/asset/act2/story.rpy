# game3 / act2 — "The Long Shelf"
#
# A second act in the dialect `act1/story.rpy` documents. Same keywords, same
# `story.rpy` name, nothing else in common: another backdrop, another cast,
# another set of lines. If pointing the asset slot at this module changes the
# story and nothing outside `asset/` had to change, the slot works.
#
# Four names appear here where act one had three. That is deliberate — a name
# is a definition, so the extra one is a spawn during reconcile and no edit
# anywhere else.

label start

scene archive
show mira
say mira "Two hundred years of weather reports, and not one of them for today."

show oskar
say oskar "You are on the wrong shelf. Today is upstairs, with the living."
say mira "Today is why I am down here. Look at the gap."

say oskar "...Nineteen years missing. Someone signed them out."
say mira "Someone signed them out and never came back up the stairs."

show keeper
say keeper "The stairs are mine, and I would have noticed."

choice
    -> "Ask the keeper." ask
    -> "Read the gap." read

label ask
say mira "Then notice for me. Who was the last name in your book?"
say keeper "The book stops nineteen years ago. I had not thought about that until now."
jump after

label read
say mira "The shelf remembers better than the book. See the dust line?"
say oskar "It steps down. Someone shelved these in the dark, in a hurry."
jump after

label after
show halden
say halden "In a hurry, yes. I was told the flood would reach the ground floor by morning."

say mira "There was no flood."
say halden "There was no morning either, for the man who told me. I have been waiting to say that to someone."

hide oskar
hide keeper
say mira "Then say the rest of it. I have two hundred years of shelf and all night."

hide halden
hide mira
scene lamplight
say mira "Act two ends with the lamp still lit."
