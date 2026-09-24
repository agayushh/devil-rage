#!/usr/bin/env python3
"""Generate src/campaign.rs. Width is fixed so maps cannot drift."""
from pathlib import Path

W = 40

class G:
    def __init__(self, h):
        self.rows = [list(" " * W) for _ in range(h)]

    def set(self, x, y, ch):
        if not (0 <= x < W and 0 <= y < len(self.rows)):
            raise SystemExit(f"oob {x},{y} {ch}")
        self.rows[y][x] = ch

    def fill(self, x0, x1, y, ch):
        for x in range(x0, x1):
            self.set(x, y, ch)

    def text(self):
        return "\n".join("".join(r) for r in self.rows)

LEVELS = []
SECRET = []
VERSUS = []

def add(bucket, w, d, s, name, hint, key, grid, traps="&[] as &'static [Trap]"):
    bucket.append(dict(w=w, d=d, s=s, name=name, hint=hint, key=key, map=grid.text(), traps=traps))

def L(*a, **k):
    add(LEVELS, *a, **k)

def floor(g, y, x0=0, x1=W):
    g.fill(x0, x1, y, "#")

# ---------- world 1 door 0: Floor ----------
g = G(2); g.set(1, 0, "@"); g.set(28, 0, "D"); floor(g, 1); g.fill(10, 15, 1, "M")
L(0, 0, 0, "A Normal Floor", "the middle is a rumor", "None", g)

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1); g.fill(6, 32, 1, "C")
L(0, 0, 1, "Please Walk Carefully", "stopping is how the floor wins", "None", g,
  "&[Trap::Delays { group: 0.22, solo: 0.50 }]")

g = G(3); g.set(1, 0, "@"); g.set(30, 0, "D"); floor(g, 1); g.fill(10, 14, 1, "F"); g.fill(10, 14, 2, "^")
L(0, 0, 2, "Instant Regret", "some floors never agreed to exist", "None", g)

g = G(2); g.set(1, 0, "@"); g.set(18, 0, "D"); floor(g, 1); g.fill(16, 20, 1, "M")
L(0, 0, 3, "The Doorstep", "arrive in the air, or don't arrive", "None", g)

g = G(3); g.set(1, 0, "@"); g.set(34, 0, "D"); floor(g, 1)
g.fill(8, 12, 1, "F"); g.fill(20, 25, 1, "M"); g.fill(8, 12, 2, "^"); g.fill(20, 25, 2, "^")
L(0, 0, 4, "Two Lies", "one falls later. one was never there", "None", g)

# ---------- world 1 door 1: Spikes ----------
g = G(2); g.set(1, 0, "@"); g.set(34, 0, "D"); floor(g, 1)
for x in (16, 17, 18):
    g.set(x, 0, ",")
L(0, 1, 0, "Say Hello", "they introduce themselves late", "None", g)

g = G(4); g.set(1, 2, "@"); g.set(36, 2, "D"); floor(g, 3)
g.fill(8, 22, 0, "v"); g.set(14, 1, "o")
L(0, 1, 1, "Free Money", "look up only if you like surprises", "None", g)

g = G(4); g.set(1, 2, "@"); g.set(36, 2, "D"); g.set(8, 2, "o"); floor(g, 3)
g.fill(4, 16, 1, "#"); g.set(10, 0, "K")
L(0, 1, 2, "The Low Road", "the short way spends you", "Some(0)", g)

g = G(2); g.set(1, 0, "@"); g.set(34, 0, "D"); g.set(12, 0, "A"); floor(g, 1); g.fill(10, 14, 1, " ")
L(0, 1, 3, "Pendulum", "wait until it looks away", "None", g)

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); g.set(14, 0, "^"); g.set(24, 0, "^"); floor(g, 1)
L(0, 1, 4, "Don't Stop", "hesitation is edible", "None", g, "&[Trap::Chase { row: 0, speed: 4.0 }]")

# ---------- world 1 door 2: Motion ----------
g = G(4); g.set(1, 2, "@"); g.set(36, 2, "D"); floor(g, 3)
L(0, 2, 0, "Coming Through", "it has somewhere to be. so do you", "None", g,
  "&[Trap::Wall { x: 22.0, y: 2.0, w: 2.0, h: 1.0, tx: 4.0, ty: 2.0, speed: 6.5, at: 6.0 }]")

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1, 0, 8); floor(g, 1, 30, 40)
L(0, 2, 1, "The Lift", "stand on it. do not outrun it", "None", g,
  "&[Trap::Platform { x: 4.0, y: 1.0, w: 6.0, range: 22.0, speed: 0.9, vertical: false }]")

g = G(2); g.set(1, 0, "@"); g.set(30, 0, "D"); floor(g, 1); g.set(7, 1, "B"); g.fill(8, 15, 1, " ")
L(0, 2, 2, "Boing", "hold the direction you mean", "None", g)

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1); g.fill(12, 24, 1, "~"); g.fill(24, 28, 1, " ")
L(0, 2, 3, "Black Ice", "enter slowly or donate yourself to the pit", "None", g)

g = G(2); g.set(6, 0, "@"); g.set(34, 0, "D"); floor(g, 1); g.fill(0, 4, 1, " "); g.fill(18, 22, 1, " ")
L(0, 2, 4, "About Face", "your first instinct is the pit", "None", g,
  "&[Trap::Reverse { at: 0.0, sticky: true }]")

# ---------- world 1 door 3: Liars ----------
g = G(5); g.set(1, 1, "@"); g.set(36, 1, "d"); floor(g, 2); g.fill(8, 12, 2, " ")
g.set(16, 3, "K"); g.set(32, 3, "D"); floor(g, 4)
L(0, 3, 0, "Not That One", "the obvious exit pays rent to the spikes", "Some(1)", g)

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); g.set(10, 0, "*"); g.set(18, 0, "o"); g.set(26, 0, "*"); floor(g, 1)
L(0, 3, 1, "Middle Child", "two of them are innocent", "None", g)

g = G(5)
g.set(4, 0, "p"); g.fill(2, 7, 1, "^"); g.set(30, 0, "q"); g.set(34, 0, "D")
g.set(2, 2, "@"); g.set(10, 2, "p"); g.set(18, 2, "q"); floor(g, 3); floor(g, 1, 28, 40)
g.fill(2, 7, 0, " "); g.set(4, 0, "p")
# rebuild row0 properly
g.rows[0] = list(" " * W)
g.set(4, 0, "p"); g.fill(2, 7, 1, "^"); g.set(30, 0, "q"); g.set(34, 0, "D"); floor(g, 1, 28, 40)
L(0, 3, 2, "Wrong Mouth", "the low circle is a rumor with teeth", "None", g)

g = G(4); g.set(1, 2, "@"); g.set(34, 1, "D"); g.set(22, 1, "K"); floor(g, 0); floor(g, 3); g.fill(20, 28, 3, "^")
L(0, 3, 3, "Upside Down", "after the line, the ceiling is the floor", "Some(2)", g,
  "&[Trap::Gravity { at: 8.0 }]")

g = G(4); g.set(1, 2, "@"); g.set(36, 2, "D"); floor(g, 3); g.fill(18, 25, 3, " ")
g.fill(3, 13, 1, "v"); floor(g, 0, 0, 16)
L(0, 3, 4, "Too Much", "the tunnel hates ambition. the pit demands it", "None", g,
  "&[Trap::HighJump { at: 14.0, scale: 1.58 }]")

# ---------- world 1 door 4: Threshold ----------
g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1); g.fill(8, 12, 1, "M"); g.set(20, 0, ","); g.set(21, 0, ",")
L(0, 4, 0, "Memory", "you have met both of these", "None", g)

g = G(4); g.set(1, 2, "@"); g.set(34, 2, "D"); g.set(22, 0, "K"); floor(g, 3); floor(g, 1, 14, 28)
L(0, 4, 1, "It Moved", "let it leave, then go where it went", "Some(3)", g,
  "&[Trap::MoveDoor { at: 12.0, tx: 20.0, ty: 0.0, speed: 16.0 }]")

g = G(2); g.set(1, 0, "@"); g.set(34, 0, "D"); floor(g, 1); g.set(6, 1, "B"); g.fill(8, 14, 1, " "); g.set(11, 0, "A")
L(0, 4, 2, "Spring Cleaning", "the arc goes over the teeth", "None", g)

g = G(2); g.set(6, 0, "@"); g.set(34, 0, "D"); g.set(16, 0, "K"); floor(g, 1); g.fill(0, 4, 1, " "); g.fill(10, 28, 1, "~")
L(0, 4, 3, "Left Foot Ice", "press the wrong key, slowly", "Some(4)", g,
  "&[Trap::Reverse { at: 0.0, sticky: true }]")

g = G(3); g.set(1, 1, "@"); g.set(36, 0, "d"); g.set(28, 1, "D"); g.set(30, 1, "K")
floor(g, 2); g.fill(14, 18, 2, "M"); g.set(8, 1, ",")
L(0, 4, 4, "Threshold", "the high door is a costume", "Some(4)", g)

# fix duplicate key 4 — stage 3 is Some(4), stage 4 should be nothing extra.
# I'll correct stage 4 key after the list is built if needed.

# ---------- world 2 door 0: Burn ----------
g = G(3); g.set(1, 1, "@"); g.set(36, 1, "D"); floor(g, 0); floor(g, 2, 0, 8); floor(g, 2, 28, 40)
L(1, 0, 0, "Pack", "hold jump. the ceiling is your friend", "None", g, "&[Trap::Jetpack(4.0)]")

g = G(4); g.set(1, 2, "@"); g.set(36, 2, "D"); g.set(18, 2, "K")
floor(g, 3, 0, 10); floor(g, 3, 14, 22); floor(g, 3, 26, 40)
g.fill(0, 7, 0, "v"); g.fill(16, 20, 0, "v"); g.fill(28, 40, 0, "v")
L(1, 0, 1, "Tap", "jump where the ceiling is bald", "Some(5)", g)

g = G(3); g.set(1, 1, "@"); g.set(36, 1, "D"); g.set(20, 0, "o"); floor(g, 0, 14, 28); floor(g, 2, 0, 12); floor(g, 2, 26, 40)
L(1, 0, 2, "Thirsty", "the coin drinks the tank", "None", g,
  "&[Trap::Jetpack(3.0), Trap::FuelDrain { x: 14.0, y: 0.0, w: 14.0, h: 2.0, rate: 6.0 }]")

g = G(3); g.set(1, 1, "@"); g.set(36, 1, "D"); floor(g, 2); g.set(0, 1, ">"); g.fill(10, 13, 1, "#"); g.fill(22, 25, 1, "#")
L(1, 0, 3, "Windows", "move when the lane is empty", "None", g)

g = G(3); g.set(1, 1, "@"); g.set(36, 1, "D"); g.set(20, 1, "K"); floor(g, 0, 0, 23)
floor(g, 2, 0, 28); floor(g, 2, 32, 40)
L(1, 0, 4, "It Quits", "land before the pack remembers it has a job", "Some(6)", g,
  "&[Trap::Jetpack(3.5), Trap::JetpackOff { at: 14.0 }]")

# ---------- world 2 door 1: Ballistics ----------
g = G(3); g.set(2, 1, "@"); g.set(36, 1, "D"); floor(g, 2); g.set(0, 1, ">"); g.set(8, 1, "#"); g.set(18, 1, "#"); g.set(28, 1, "#")
L(1, 1, 0, "Cover", "the blocks are the only honest things here", "None", g)

g = G(3); g.set(2, 1, "@"); g.set(34, 1, "D"); floor(g, 2); g.set(38, 1, "X"); g.fill(16, 20, 2, " ")
L(1, 1, 1, "Sideways", "down is a direction with options", "None", g)

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); g.set(22, 0, "K"); g.set(12, 0, "^"); g.set(26, 0, "^"); floor(g, 1)
L(1, 1, 2, "Closer", "faster than the last one. still slower than you", "Some(7)", g,
  "&[Trap::Chase { row: 0, speed: 5.4 }]")

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1); g.fill(12, 20, 1, "C"); g.set(0, 0, ">")
L(1, 1, 3, "Both", "the floor and the bullet want the same thing", "None", g,
  "&[Trap::Delays { group: 0.2, solo: 0.28 }]")

g = G(3); g.set(1, 1, "@"); g.set(8, 1, "q"); g.set(34, 1, "q"); g.set(36, 1, "D"); floor(g, 2); g.set(0, 1, ">"); g.fill(4, 7, 1, "#")
# q pair: first at (8,1), second at (34,1) if 8 is left of 34. Reading order: both on row1, col 8 then 34. Good.
# Wait I set 34 first in the code but the map is spatial, reading order is by position not call order. Good.
L(1, 1, 4, "Leave", "the circle is the exit. the gun is the lesson", "None", g)

# ---------- world 2 door 2: House Rules ----------
g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); g.set(18, 0, "K"); floor(g, 1); g.fill(10, 16, 1, "G"); g.fill(30, 34, 1, " ")
L(1, 2, 0, "One", "the first hole is a rumor. the second is not", "Some(8)", g, "&[Trap::JumpLimit(1)]")

g = G(5); g.set(1, 3, "@"); g.set(36, 3, "D"); floor(g, 4); floor(g, 0); g.fill(8, 14, 2, "v"); g.fill(20, 28, 2, "v")
L(1, 2, 1, "Optional Ground", "tap. the ceiling is keeping score", "None", g, "&[Trap::Infinite { at: 0.0 }]")

g = G(3); g.set(1, 1, "@"); g.set(6, 1, "+"); g.set(22, 1, "-"); g.set(36, 1, "D")
floor(g, 2); g.fill(13, 14, 2, " "); floor(g, 0, 10, 18); g.set(30, 1, "#"); g.set(32, 1, "#")
L(1, 2, 2, "Manners", "wide for the hole, small for the slot", "None", g)

g = G(4); g.set(2, 2, "@"); g.set(8, 2, "t"); g.set(12, 2, "n"); g.set(36, 2, "D"); floor(g, 3); floor(g, 0, 0, 16); floor(g, 1, 20, 40)
L(1, 2, 3, "Promotion", "the button is not a gift", "None", g)

g = G(3); g.set(1, 1, "@"); g.set(34, 1, "D"); g.set(16, 1, "K"); floor(g, 2); g.fill(8, 12, 2, "M"); g.set(22, 1, "o"); g.set(0, 1, ">")
L(1, 2, 4, "House Rules", "you know every one of these. they know you", "Some(9)", g,
  "&[Trap::Delays { group: 0.18, solo: 0.4 }]")

# ---------- world 3 door 0: Company ----------
g = G(2); g.set(18, 0, "@"); g.set(34, 0, "D"); floor(g, 1); g.set(11, 0, "^"); g.set(28, 0, "^")
L(2, 0, 0, "Plus One", "whatever you survive, they have to survive too", "None", g, "&[Trap::Clone(CloneKind::Mirror)]")

g = G(2); g.set(18, 0, "@"); g.set(34, 0, "D"); g.set(26, 0, "o"); g.set(13, 0, "^"); g.set(30, 0, "K"); floor(g, 1)
L(2, 0, 1, "Shared Appetite", "jump the snack. your shadow has a spike", "Some(10)", g, "&[Trap::Clone(CloneKind::Mirror)]")

g = G(2); g.set(10, 0, "@"); g.set(24, 0, "D"); g.set(14, 0, "^"); g.set(30, 0, "2"); g.set(36, 0, "^"); floor(g, 1)
L(2, 0, 2, "Opposite Day", "you go to the door. they go the other way", "None", g, "&[Trap::Clone(CloneKind::Opposite)]")

g = G(8)
g.set(1, 2, "2"); g.set(30, 2, "E"); floor(g, 3); g.fill(14, 16, 3, " ")
g.set(1, 6, "@"); g.set(30, 6, "D"); floor(g, 7); g.fill(14, 16, 7, " ")
L(2, 0, 3, "Same Buttons", "two floors, one cowardice", "None", g, "&[Trap::Clone(CloneKind::Shadow)]")

g = G(5)
g.set(1, 1, "2"); g.set(34, 1, "E"); g.set(18, 1, "K"); floor(g, 2); g.fill(14, 17, 2, "M")
g.set(1, 3, "@"); g.set(34, 3, "D"); floor(g, 4); g.fill(14, 17, 4, "M")
L(2, 0, 4, "Both Doors", "neither of you is allowed to be the hero alone", "Some(11)", g,
  "&[Trap::Clone(CloneKind::Dual), Trap::Delays { group: 0.55, solo: 0.4 }]")

# ---------- world 3 door 1: Glitch ----------
g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1); g.fill(3, 7, 1, "f")
L(2, 1, 0, "Blink", "it is solid if you do not think about it", "None", g, "&[Trap::FlickerPeriod(1.35)]")

g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1); g.fill(10, 28, 1, " "); g.fill(10, 28, 1, "G"); g.fill(12, 18, 1, "F")
# F overwrites some ghosts — the visible fake is a trap, the rest of the gap is ghost. 
# Wait fill G then F overwrites 12-18. Walking on F falls. The ghost is 10-12 and 18-28. 
# A jump from 8 across F (12-18 is 6 tiles) might land on ghost. Walking onto F at 12 falls.
# Simpler: entire gap is ghost, looks empty. They must walk onto nothing.
g = G(2); g.set(1, 0, "@"); g.set(36, 0, "D"); floor(g, 1, 0, 8); floor(g, 1, 30, 40); g.fill(8, 30, 1, "G")
L(2, 1, 1, "Unlisted", "the gap filed paperwork", "None", g)

g = G(3); g.set(1, 1, "@"); g.set(36, 1, "D"); g.set(18, 1, "K"); floor(g, 2); g.set(12, 1, ","); g.set(13, 1, ",")
L(2, 1, 2, "Late Edit", "the door relocates. the spikes do too", "Some(12)", g,
  "&[Trap::MoveDoor { at: 8.0, tx: 28.0, ty: 1.0, speed: 14.0 }]")

g = G(3); g.set(0, 0, ">"); g.set(2, 1, "@"); g.set(36, 1, "D"); floor(g, 2); g.fill(8, 11, 2, "M"); g.set(20, 1, "o")
L(2, 1, 3, "Stack", "nothing here is new. that is the problem", "None", g,
  "&[Trap::Delays { group: 0.55, solo: 0.42 }]")

g = G(3); g.set(2, 1, "@"); g.set(36, 1, "d"); g.set(8, 1, "D"); floor(g, 2); g.fill(16, 22, 2, "C")
L(2, 1, 4, "Goodbye", "the far door is the credits. the near door is the door", "None", g,
  "&[Trap::Delays { group: 0.2, solo: 0.36 }, Trap::Taunt { at: 24.0, text: \"almost\" }]")

# ---------- secret ----------
g = G(2); g.set(8, 0, "@"); g.set(34, 0, "D"); floor(g, 1); g.fill(0, 6, 1, " "); g.set(4, 0, "^")
add(SECRET, 9, 0, 0, "Read the Footer", "the tip at the bottom is the trap", "None", g,
    "&[Trap::Reverse { at: 0.0, sticky: true }, Trap::Lie]")

g = G(3); g.set(1, 1, "@"); g.set(36, 1, "D"); floor(g, 2); g.fill(10, 13, 2, "M"); g.set(20, 1, "o"); g.set(22, 1, ",")
add(SECRET, 9, 0, 1, "Greatest Hits", "you have died to all of these", "None", g,
    "&[Trap::Delays { group: 0.55, solo: 0.42 }]")

g = G(3); g.set(2, 1, "@"); g.set(28, 1, "D"); floor(g, 2); floor(g, 0, 26, 32)
add(SECRET, 9, 0, 2, "For Once", "walk. it is only a door", "None", g, "&[Trap::Peace, Trap::NoSkip]")

# ---------- versus ----------
def V(name, hint, grid, traps="&[] as &'static [Trap]"):
    add(VERSUS, 8, 0, len(VERSUS), name, hint, "None", grid, traps)

g = G(2); g.set(2, 0, "@"); g.set(4, 0, "1"); g.set(36, 0, "D"); floor(g, 1); g.fill(14, 20, 1, "M")
V("Shared Floor", "the bridge falls for both of you", g)

g = G(2); g.set(2, 0, "@"); g.set(4, 0, "1"); g.set(36, 0, "D"); floor(g, 1)
for x in (16, 17, 22, 23):
    g.set(x, 0, ",")
V("Same Surprise", "whoever runs first wakes them", g)

g = G(2); g.set(4, 0, "@"); g.set(6, 0, "1"); g.set(34, 0, "D"); floor(g, 1); g.fill(0, 3, 1, " ")
V("Both Wrong", "the pit is behind the instinct", g, "&[Trap::Reverse { at: 0.0, sticky: true }]")

g = G(2); g.set(2, 0, "@"); g.set(4, 0, "1"); g.set(36, 0, "D"); g.set(18, 0, "A"); floor(g, 1); g.fill(14, 24, 1, " ")
V("One Saw", "it does not take turns", g)

g = G(2); g.set(2, 0, "@"); g.set(5, 0, "1"); g.set(36, 0, "D"); g.set(12, 0, "!"); g.set(18, 0, "o"); floor(g, 1)
V("Betrayal", "the button is not for you", g)

# ---- key audit ----
# Stage 0,4,4 was given Some(4) AND 0,4,3 is Some(4). Fix 0,4,4 to None and remove its K? 
# Threshold should keep a key. Left Foot Ice can drop the key.
for lv in LEVELS:
    if lv["w"] == 0 and lv["d"] == 4 and lv["s"] == 3:
        lv["key"] = "None"
        lv["map"] = lv["map"].replace("K", " ")

ids = []
for lv in LEVELS:
    if lv["key"] != "None":
        ids.append(lv["key"])
expect = [f"Some({i})" for i in range(13)]
if ids != expect:
    raise SystemExit(f"keys {ids} != {expect}")

def rust_level(lv):
    m = lv["map"].replace("\\", "\\\\").replace("\"", "\\\"")
    body = "\\n".join(m.split("\n"))
    return f'''    LevelDef {{
        world: {lv["w"]},
        door: {lv["d"]},
        stage: {lv["s"]},
        name: "{lv["name"]}",
        hint: "{lv["hint"]}",
        key: {lv["key"]},
        map: "{body}",
        traps: {lv["traps"]},
    }},'''

def chunk(name, items):
    inner = "\n".join(rust_level(lv) for lv in items)
    return f"pub static {name}: &[LevelDef] = &[\n{inner}\n];\n"

src = f'''//! Original stages. Same loop as Level Devil — doors of five, three worlds,
//! keys, a true door, two player — written for a terminal, not copied from Unept.

use crate::level::{{LevelDef, Trap}};
use crate::model::{{CloneKind, Save, Step, KEY_TOTAL}};

pub struct WorldInfo {{
    pub name: &'static str,
    pub tag: &'static str,
    pub doors: &'static [&'static str],
}}

pub static WORLDS: &[WorldInfo] = &[
    WorldInfo {{
        name: "Level Devil",
        tag: "every floor is a rumor",
        doors: &["Floor", "Spikes", "Motion", "Liars", "Threshold"],
    }},
    WorldInfo {{
        name: "Level Devil-er",
        tag: "the rules walked out",
        doors: &["Burn", "Ballistics", "House Rules"],
    }},
    WorldInfo {{
        name: "Level Devil-est",
        tag: "bring someone to die with",
        doors: &["Company", "Glitch"],
    }},
];

{chunk("CAMPAIGN", LEVELS)}
{chunk("SECRET", SECRET)}
{chunk("VERSUS", VERSUS)}

pub fn find(world: u8, door: u8, stage: u8) -> Option<&'static LevelDef> {{
    CAMPAIGN.iter().find(|l| l.world == world && l.door == door && l.stage == stage)
}}

pub fn door_count(world: u8) -> usize {{
    CAMPAIGN.iter().filter(|l| l.world == world).map(|l| l.door as usize).max().map(|d| d + 1).unwrap_or(0)
}}

pub fn stage_count(world: u8, door: u8) -> usize {{
    CAMPAIGN.iter().filter(|l| l.world == world && l.door == door).count()
}}

pub fn bump(save: &mut Save) -> Step {{
    if save.cleared || save.world >= 3 {{
        save.cleared = true;
        save.world = 3;
        return Step::Done;
    }}
    let stages = stage_count(save.world, save.door) as u8;
    if stages == 0 {{
        save.cleared = true;
        return Step::Done;
    }}
    save.stage += 1;
    if save.stage < stages {{
        return Step::Stage;
    }}
    save.stage = 0;
    let doors = door_count(save.world) as u8;
    save.door += 1;
    if save.door < doors {{
        return Step::Door;
    }}
    save.door = 0;
    save.world += 1;
    if save.world < 3 {{
        return Step::World;
    }}
    save.cleared = true;
    save.world = 3;
    save.door = 0;
    save.stage = 0;
    Step::Done
}}

pub fn check_all() -> Result<(), String> {{
    use crate::sim::Sim;
    for def in CAMPAIGN.iter().chain(SECRET.iter()) {{
        Sim::boot(def, &[], false)?;
    }}
    for def in VERSUS.iter() {{
        Sim::boot(def, &[], true)?;
    }}
    if CAMPAIGN.len() != 50 {{
        return Err(format!("expected 50 campaign levels, got {{}}", CAMPAIGN.len()));
    }}
    for w in 0..3 {{
        let doors = door_count(w);
        if doors != WORLDS[w as usize].doors.len() {{
            return Err(format!("world {{w}} door mismatch"));
        }}
        for d in 0..doors {{
            if stage_count(w as u8, d as u8) != 5 {{
                return Err(format!("world {{w}} door {{d}} is not 5 stages"));
            }}
        }}
    }}
    let mut keys: Vec<u8> = CAMPAIGN.iter().filter_map(|l| l.key).collect();
    keys.sort_unstable();
    if keys != (0..KEY_TOTAL).collect::<Vec<_>>() {{
        return Err(format!("keys {{keys:?}}"));
    }}
    Ok(())
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use crate::sim::rehearse;

    #[test]
    fn structure() {{
        check_all().unwrap();
    }}

    #[test]
    fn bump_through_first_door() {{
        let mut s = Save::default();
        assert_eq!(bump(&mut s), Step::Stage);
        assert_eq!((s.world, s.door, s.stage), (0, 0, 1));
        for _ in 0..3 {{
            bump(&mut s);
        }}
        assert_eq!(bump(&mut s), Step::Door);
        assert_eq!((s.world, s.door, s.stage), (0, 1, 0));
    }}

    #[test]
    fn rehearse_opening() {{
        rehearse(find(0, 0, 0).unwrap(), &[(1.05, 1, false), (0.75, 1, true), (1.6, 1, false)]).unwrap();
    }}

    #[test]
    fn rehearse_keep_moving() {{
        rehearse(find(0, 0, 1).unwrap(), &[(3.2, 1, false)]).unwrap();
    }}

    #[test]
    fn rehearse_reverse() {{
        rehearse(find(0, 2, 4).unwrap(), &[(1.15, -1, false), (0.7, -1, true), (1.6, -1, false)]).unwrap();
    }}

    #[test]
    fn rehearse_ignore_the_coin() {{
        rehearse(find(0, 1, 1).unwrap(), &[(3.2, 1, false)]).unwrap();
    }}

    #[test]
    fn rehearse_gravity() {{
        rehearse(find(0, 3, 3).unwrap(), &[(4.6, 1, false)]).unwrap();
    }}

    #[test]
    fn rehearse_jetpack() {{
        rehearse(find(1, 0, 0).unwrap(), &[(0.25, 1, false), (2.8, 1, true)]).unwrap();
    }}

    #[test]
    fn fuzz_does_not_explode() {{
        use crate::sim::{{Sim, Sticks}};
        for def in CAMPAIGN.iter().chain(SECRET.iter()) {{
            let mut sim = Sim::boot(def, &[], false).unwrap();
            for f in 0..100 {{
                let jump = f % 50 == 8;
                let dir = if f % 90 < 50 {{ 1 }} else {{ -1 }};
                sim.update(1.0 / 60.0, &Sticks::one(dir, jump, jump));
                assert!(sim.actors[0].x.is_finite(), "{{}}", def.name);
            }}
        }}
        for def in VERSUS {{
            let mut sim = Sim::boot(def, &[], true).unwrap();
            sim.update(0.5, &Sticks::one(1, false, false));
            assert!(sim.actors.len() >= 2, "{{}}", def.name);
        }}
    }}
}}
'''

# The f-string above doubled braces for rust format. But I also used {lv} in rust_level
# before the f-string... rust_level is called inside chunk() which is called inside the f-string.
# WAIT. The whole src is an f-string. rust_level returns braces. Those will be interpreted
# by the f-string if chunk() is interpolated... chunk() is called with {chunk(...)} so the
# RETURN value of chunk is inserted, and braces inside the return value are NOT re-parsed.
# Good.
# But check_all's rust format strings I wrote as {{}} so they become {}. Good.
# The test module uses {{ }} as well. Good.
# WORLD static uses no format. Good.

# Problem: rust_level is NOT inside the f-string evaluation of braces from its result.
# chunk() returns a plain string inserted into the f-string. Braces in that string stay.
# Good.

Path("src/campaign.rs").write_text(src)
print(f"campaign {len(LEVELS)} secret {len(SECRET)} versus {len(VERSUS)}")
