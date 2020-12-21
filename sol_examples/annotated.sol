[global]
name = Klondike (hard)
# Number of decks: 1 or 2
decks = 1

# Describes a pile of extra face-down cards as in left-top corner in Klondike.
# Some solitaries do noty have it.
[deck]
# 'unlimited' or number of redeals. Used only if 'deal_to' is 'waste'
redeals = unlimited
# Number of cards moved to waste per click.
# If deal_to is 'play area', the number always equals the number of columns in play area.
deal_by = 3
# Where to move cards from the deck: to waste or to play area columns.
# 'waste', 'deck', and 'side' = to waste
# 'columns' = to play area columns
deal_to = waste

# Must be at least one slot (and no more than 8 slots)
# From leftmost to rightmost column.
[foundation]
# All column properties are mandatory:
# First pair is what card can be put to the column when it is empty
# 1. First card face - card face, or 'any' to allow starting the foundation from any card
# 2. First card suit - 'any' to start from any suit
# Second pair is in what order next cards must be placed to the foundation
# 3. Face order - 'ascending' or 'asc' in ascending order(from 'A' to 'K'),
#    'desc' or 'descending' in descending order (from 'K' to 'A'),
#    'any' when the next card must have the next or previous face value
# 4. Suit order - 'same' or 'same suit' - cards must have the same suit,
#    'same color' - cards must have the same suit color (only spades+clubs or diamonds+hearts),
#    'alternate' - cards must have alternate colors,
#    'any' - no suit restrictions, put a card of any suit.
column = A, any, ascending, same suit
column = A, any, ascending, same suit
column = A, any, ascending, same suit
column = A, any, ascending, same suit

# Free cells configuration: most of solitaries do not have it.
#[temp]
# the number of free cells: no more than 4 slots
# slots = 0

# Column area configuration
[column]
# What cards can be moved to another column:
#   'top' - only the top card of a pile
#   'ordered' - a few top cards of a pile can be moved only of they follow colum sort orders
#   'any' - any pile of face-up cards can be moved to another pile
playable_card = any
# What card face can start the pile if it gets empty:
#   'any' - any card can start the pile
#   CARD_FACE - only card with this face can put when the pile is empty
#   'none' or 'unavail' - if a pile gets empty, no card can be put to it
refill = K
# Order of cards in all play area columns. A pair of values:
# 1. Face order - 'ascending' or 'asc' in ascending order(from 'A' to 'K'),
#    'desc' or 'descending' in descending order (from 'K' to 'A'),
#    'any' when the next card must have the next or previous face value
# 2. Suit order - 'same' or 'same suit' - cards must have the same suit,
#    'same color' - cards must have the same suit color (only spades+clubs or diamonds+hearts),
#    'alternate' or 'alternate color' - cards must have alternate colors,
#    'any' - no suit restrictions, put a card of any suit
#    'none' or 'disable' - no card can be put to this pile from another pile. Useful to 
#           create "take-only" prefilled columns (like the first one in 'American toad')
order = descending, alternate color
# Intial column configuration. Contains two or three values:
# 1. Total number of cards
# 2. Number of face-up cards (must be at least 1 if a pile is not empty)
# 3. Optional value with the single choice: 'take' or 'take only' - cards can be put from
#    this pile anywhere, but putting cards to this pile is forbidden. Used in few solitaries,
#    e.g, the first play area column in 'American toad'.
column = 1, 1
column = 2, 1
column = 3, 1
column = 4, 1
column = 5, 1
column = 6, 1
column = 7, 1
