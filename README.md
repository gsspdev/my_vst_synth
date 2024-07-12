# Here's how to package the VST plugin on MacOS:

## Build the plugin
```bash
cargo build --release
```
## Create a VST bundle structure:

```bash
# Create the bundle directory structure
mkdir -p "MyVSTSynth.vst/Contents/MacOS"

# Copy the compiled library into the bundle
cp target/release/libmy_vst_synth.dylib "MyVSTSynth.vst/Contents/MacOS/MyVSTSynth"

# Create an Info.plist file
cat << EOF > "MyVSTSynth.vst/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>MyVSTSynth</string>
    <key>CFBundleIdentifier</key>
    <string>com.yourcompany.MyVSTSynth</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>MyVSTSynth</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CSResourcesFileMapped</key>
    <true/>
</dict>
</plist>
EOF
```

## You can now copy the entire MyVSTSynth.vst bundle to your VST plugins folder. The default locations for VST plugins on macOS are:
▪	/Library/Audio/Plug-Ins/VST (system-wide)
▪	~/Library/Audio/Plug-Ins/VST (user-specific)

## Copy to user-specific VST folder
```bash
cp -R MyVSTSynth.vst ~/Library/Audio/Plug-Ins/VST/
```
## After copying, you may need to restart your DAW or rescan for plugins.
This process creates a proper .vst bundle that macOS and most DAWs will recognize as a VST plugin. The .dylib file is contained within this bundle structure, but the DAW interacts with the .vst bundle as a whole.