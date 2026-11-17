## Downloads

| Platform | File |
|---|---|
| macOS (Apple Silicon + Intel) | `Oneko-<version>.dmg` |
| Windows 10/11 (x64) | `oneko-windows-x64.exe` |

Both builds are unsigned, so each OS shows a one-time warning. Steps below.

## macOS

Open the dmg, drag Oneko to Applications. The app isn't notarized (that needs Apple's paid developer program),
so macOS will say **"Oneko" Not Opened, Apple could not verify...** the first time. One-time fix:

1. Click **Done** (not Move to Trash).
2. Open **System Settings, Privacy & Security**, scroll to *Security*: you'll see *"Oneko" was blocked to protect your Mac*. Click **Open Anyway** and confirm.

Or skip the dialog entirely from Terminal:

```sh
xattr -dr com.apple.quarantine /Applications/Oneko.app
```

## Windows

Run the exe. SmartScreen will warn because it isn't code-signed: click **More info**, then **Run anyway**. One time only.

## Using it

Click the cat in the menu bar / system tray for Pause (hide), Character (neko, tora, dog, sakura, tomoyo) and Quit.
