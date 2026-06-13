# SSBU Online Deluxe

A performance and online enhancement mod for **Super Smash Bros. Ultimate** that introduces latency controls, render optimizations, and real-time online information.

> ⚠️ This is a work in progress. Features and stability may change as development continues.  
> ⚠️ Use at your own risk. I have been testing this mod online personally without any major issues, but there is still a non-zero risk of a ban. The overclocks are intentionally minimal; however, any hardware damage or account penalties remain your responsibility.  

## ✅ Compatibility

- ✔️ Nintendo Switch (console)
- ✔️ Eden Emulator (requires workaround, see installation section below)
- ⚠️ Other emulators: not yet tested (not planned)
- ⚠️ HDR support not yet tested (planned)

## 📦 Installation

- Remove any previous latency slider mod, vsync mod, and less lag mod:
- Download and extract the zip from the releases, then:
- Copy the `atmosphere` folder to the root of the SD card (or sdmc directory on emulator).
- Your Switch may need a full restart for the mod to load correctly.
- Eden emulator currently requires a workaround:
  - Right click SSBU -> Click `Configure Game` -> Click `System` tab -> Check `RNG Seed` -> Set to `00000000`

> ⚠️ The latest skyline currently causes a crash. Use the skyline files bundled in with the release zip.  
> ⚡ If you are using emulator, I recommend using [yuzu-ssbu-optimizer](https://github.com/saad-script/yuzu-ssbu-optimizer/releases). It will setup everything for you.


## 🎮 Controls

### Native UI (Online Character Select Screen and Online Arena)

- On the character select screen or online arena:
  - `D-pad Left/Right`: Select network latency
  - `D-pad Up/Down`: Select render profile

- On the character select screen (more than one opponent):
  - `ZL + ZR + Dpad Left/Right`: Cycle between which opponent's network info to show

See 'Features' section below to see what these options do

### Overlay UI (Optional)

- `ZL + ZR + D-Pad Down` → Cycle between current window mode
  - Window Modes: `Hidden`, `Full Info`, `Performance Info`
  - In `Performance Info Mode`:
    - `D-Pad Up / Down` → Select option
    - `D-Pad Left / Right` → Change value

See 'Features' section below to see what these options do


## ✨ Features

### 🌐 Online Enhancements

- Display **opponent ping** in all online modes (including Elite Smash):
  - Network RTT (ping) / connection quality
- Show **extended opponent info** *(only if both players have the mod)*:
  - Opponent’s current network/render settings (latency slider, buffer mode, vsync status, etc.)

### 🎛️ Online Latency Controls
*(Available in Arena and Local Online modes only)*

- Adjust:
  - Latency value: Applied when entering a valid online match

### 🎛️ Render Profile Controls (for less input latency)
*(Available in Offline, Online Arena and Local Online modes only)*

**I would highly recommend you to leave these options as is. The best settings are already applied automatically when entering/exiting a valid online match. These are for those who want to experiment and try out different settings**
- Adjust:
  - Render Profile:
    - Auto: Applies the recommended profile based on platform (console/emulator) and number of players.
    - Vanilla: This is the default vanilla profile that the game uses by default.
    - LessLag: This applies optimizations to cut 3 frames of native input delay.
    - LLUltra: This applies optimizations to cut 4 frames of native input delay.
      - This also works on console, but the game resolution will be scaled down to keep it stutter free.
      - On console, you may notice that certain UI elements look glitcy, such as the fighter cut-in screen, and match start countdown ui.
    - LLDoubles (Recommended for doubles): This applies optimizations to cut 2 frames of native input delay. This should work even in doubles when there are alot of players on screen without stuttering.

Recommended for console:
  - LessLag or LessLagUltra (depending on preference)

Recommended for emulator:
  - LLUltra

Recommended for doubles:
  - LLDoubles

> Applies **selected render settings automatically** when entering a valid online match.  
> Reverts to **vanilla settings** after exiting  

## 📝 Notes and Contribution

- The dynamic resolution logic currently only applies to zoom in moves (final hit/critical hit) and Sephiroth's gigaflare.  
- Contributions are open especially for applying dynamic resolution to moves that cause stutter. I don't know if I'll have time to optimize every single move, so if you notice a specific move causes stutters, you can use smashline's api to contribute and optimize the move. You can start by viewing how `src/perf_scaler` currently applies dynamic resolution optimization.

## 🙌 Credits

Huge thanks to the following people who made this possible. Without these people, this project wouldn't have been possible:

- **Bludev**
  For SSBU render system research and the initial less-lag and latency slider mod.

- **BlankMauser**
  Creator of the SsbuSync and smash-ultelier mod, which this mod uses to modify ssbu's render system.
  BlanksMauser's work and guidance on SSBU’s rendering internals were critical to making this mod possible.

- **Kinnay** & contributors of the NintendoClients repo/wiki
  For guidance on network service implementation. The network service wouldn't have been possible if not for the incredible efforts of these people.

- **Coolsonickirby**
  For the imgui-smash plugin, making UI development significantly easier

- **The HDR team**
  For smashline, allowing for easy figher/effect/moveset hooks and adjustments 

- **The developers of Skyline**
  For the modding environment, allowing for code hooking/edits

