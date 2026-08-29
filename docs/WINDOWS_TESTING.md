# Windows testing

Use this checklist on both Windows 10 and Windows 11 before a release. Test the unsigned CI artifact for pull requests and a signed installer from the protected release draft for public releases. CI can package and validate Authenticode signatures; it cannot verify real desktop layering, audio devices, multi-monitor behavior, or user-notification delivery.

## Test setup

- Download the `virtual-pet-windows-unsigned-*` artifact from the successful CI run.
- Extract the artifact, record the Windows edition/build, display resolutions and scaling values.
- Run on a normal user account. Windows may warn about unsigned development builds; do not publish them as releases.
- Repeat the installation checks on a clean Windows VM or machine when possible.
- Record the CI run URL and installer SHA-256 in the release evidence. Do not mark the GitHub draft public until this checklist is complete.

## Manual checklist

### Window and overlay

- [ ] The pet starts as a transparent, borderless overlay with no background rectangle.
- [ ] The pet remains above normal desktop applications and does not steal keyboard focus.
- [ ] The pet does not appear as a separate taskbar button.
- [ ] Clicking through normal pet movement does not interfere with the active application.
- [ ] Chrome F11 fullscreen and Windows Terminal fullscreen show the pet above the application.
- [ ] The pet behaves predictably over borderless-fullscreen applications. Record exclusive-fullscreen exceptions, because Windows can place those applications above overlays.
- [ ] Pet movement remains inside the usable desktop area and does not hide behind the taskbar.

### Multiple monitors and DPI

- [ ] Test one monitor at 100%, 125%, 150% and 200% scaling where available.
- [ ] Test two monitors with different scaling values.
- [ ] Place a secondary monitor above and to the left of the primary monitor to exercise negative desktop coordinates.
- [ ] Move the pointer between every monitor and confirm the pet follows smoothly across boundaries.
- [ ] Confirm the pet keeps the configured size and is not clipped, stretched or blurred after crossing monitors.
- [ ] Disconnect and reconnect a monitor while the app is running; the pet must return to an available display.
- [ ] Change the primary monitor and restart the app; saved settings must still load.

### Tray and lifecycle

- [ ] The paw icon appears in the notification area, including the hidden-icons overflow.
- [ ] Every tray action performs the matching behavior: Auto, Call over, Play, Bark, Sleep and Settings.
- [ ] The default `Ctrl+Shift+P` shortcut and Settings visibility button both hide and restore the pet.
- [ ] Bark plays only when Bark is selected.
- [ ] Sleep sends the pet to its hut, starts sleep animation/audio and Wake returns it to normal behavior.
- [ ] Closing Settings does not quit the pet.
- [ ] The tray Quit action closes the pet, settings window and background process.
- [ ] Starting the app again does not create duplicate tray icons or orphan processes.

### Settings and care

- [ ] Change pet type, size, speed, volume, mute and reduced-motion settings; each change takes effect correctly.
- [ ] Close and restart the app; all settings persist.
- [ ] Feed, Pet, Play, Wash, Rest and Wake update only the expected care values and behavior.
- [ ] Hunger, energy, happiness and cleanliness remain within their valid ranges.
- [ ] Care state persists after restart and advances without freezing or excessive CPU use.
- [ ] Reduced motion avoids rapid or unnecessary animation while preserving usable controls.

### Pet packs

- [ ] Puppy, cat and red fox appear in the pet selector and load without a network connection.
- [ ] Switch between packs repeatedly; the correct atlas, bark and sleep audio load every time.
- [ ] Idle, walk, run, play, bark and sleep use valid frames without flicker or transparent flashes.
- [ ] A missing or invalid pack fails safely and does not prevent a bundled valid pack from loading.
- [ ] Preview pet packs shows side, front, back and diagonal choices without loading arbitrary URLs.
- [ ] Backup export/import restores settings, inventory and care; pack export copies only validated referenced assets.
- [ ] A sample behavior extension runs only allowlisted actions and returns to Auto.

### Optional AI

- [ ] Core movement, care, packs and audio work with AI disabled and with Ollama absent.
- [ ] Enabling AI accepts only a loopback Ollama endpoint and failure does not freeze pet movement.
- [ ] Suggested pet actions run only after pressing the confirmation button.
- [ ] Memory, proactive suggestions and voice output can each be disabled independently.

### Audio and notifications

- [ ] Bark audio plays once for the Bark action and does not loop or trigger during unrelated behavior.
- [ ] Snoring loops only while sleeping and stops immediately after Wake or Quit.
- [ ] Volume changes apply to bark and sleep audio; mute silences both.
- [ ] Audio continues to work after changing the Windows default output device.
- [ ] Care notifications, when enabled, appear in Windows Notification Center at the intended threshold.
- [ ] Disabling care notifications prevents new notifications.

### Install and uninstall

- [ ] The NSIS installer completes for a normal user without administrator access unless Windows policy requires it.
- [ ] The Start menu entry launches the installed app and shows the correct name and icon.
- [ ] Installing the same version again repairs or replaces the installation without duplicates.
- [ ] Upgrading from the previous version keeps compatible settings and care state.
- [ ] Apps & features lists Virtual Pet with the expected version and publisher information.
- [ ] Uninstall removes application binaries and shortcuts and leaves no running process.
- [ ] User settings are either preserved for reinstall or removed through an explicitly documented user choice; record the observed behavior.

## Report failures

Include the app commit, Windows edition/build, monitor layout, scaling, installation type, reproduction steps, expected and actual behavior, logs, and a short screen recording when the failure is visual. Never attach API keys, personal paths or other secrets.

## Release evidence gate

Attach this completed checklist, the CI run URL, installer SHA-256, and a maintainer approval to the release discussion or tracking issue. A successful unsigned or signed CI build is necessary but does not substitute for this Windows 10/11 GUI test.
