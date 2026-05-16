# VoxelVerse V1 Audio And Game Feel

Game feel is not decoration. It is how the player knows the world is alive.

## V1 feel goals

Every frequent action must have:

- immediate visual response;
- immediate audio response;
- short animation;
- clear success/failure state;
- no input delay;
- no mystery silence.

## Core feedback actions

Required feedback events:

- tool swing;
- block hit;
- block crack stage change;
- block break;
- block place;
- blocked mining;
- invalid placement;
- item pickup;
- inventory move;
- crafting success;
- crafting blocked;
- station open/close;
- UI hover/click;
- footstep by surface.

## Audio event architecture

Gameplay emits semantic feedback events. Audio maps them to sounds.

Good:

```text
Gameplay: BlockHit { material: Stone, strength: 0.8 }
Audio: choose stone_hit variation, pitch by strength
Renderer: play hit animation and crack pulse
```

Bad:

```text
Mining code directly plays random file path and pokes renderer animation.
```

## Sound variation

V1 should avoid robotic repetition.

Use:

- 3 to 6 variations per frequent sound if assets exist;
- small pitch variation;
- volume by strength;
- material-specific sound classes;
- cooldown to prevent audio spam.

## Mining feel

Mining must feel like striking matter.

Feedback stack:

1. hand/tool swing starts;
2. hit sound occurs at impact timing;
3. tiny camera/tool impulse;
4. crack overlay updates;
5. small particles if available;
6. block break sound and pop;
7. drop pickup or item notice.

The strike rhythm must be tunable by tool.

## Placing feel

Placing block should feel crisp.

Required:

- placement preview or clear selected face;
- short hand motion;
- material sound;
- block appears instantly;
- hotbar count updates;
- invalid place gives soft deny feedback.

## Footsteps

V1 footsteps should use block sound kind:

- grass;
- stone;
- wood;
- sand;
- snow;
- dirt;
- water if included.

Footsteps must respect movement speed and not trigger while airborne.

## UI feel

UI sounds must be subtle.

Required:

- hover optional and very light;
- click;
- move stack;
- craft success;
- error;
- open/close inventory;
- station open/close.

No casino machine noise. The interface should whisper, not juggle spoons.

## Animation rules

Animations should be short and responsive.

Targets:

- tool swing under 300 ms for basic tools unless heavy;
- hit impulse under 120 ms;
- UI slot feedback under 180 ms;
- panel open under 180 ms;
- notice fade readable but not slow.

## Feel diagnostics

For tuning, debug overlay should expose:

- selected item/tool;
- mining cooldown;
- last feedback event;
- block damage fraction;
- sound event count this frame;
- current footstep surface;
- first-person animation state.

## Feel V1 gate

Audio and feel are V1-ready when:

- mining is fun with eyes closed from audio rhythm alone;
- hitting wrong material sounds wrong in a useful way;
- placing blocks feels crisp;
- inventory actions feel responsive;
- no frequent action is silent;
- sounds do not spam or overlap absurdly;
- feedback events are centralized enough to tune.
