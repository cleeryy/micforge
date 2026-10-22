; Yeti Voice — minimal Inno Setup script.
; Bundles NO audio driver: VB-CABLE is installed separately by the user.
#define MyAppName "Yeti Voice"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "cleeryy"

[Setup]
AppId={{3B7E2A1C-8F4D-4E6A-9C2B-1A5F7D3E9B01}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\Yeti Voice
ArchitecturesAllowed=x64compatible
PrivilegesRequired=lowest
OutputDir=..\installer-output
OutputBaseFilename=YetiVoice-Setup-{#MyAppVersion}
Compression=lzma
SolidCompression=yes

[Files]
Source: "..\publish\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{autoprograms}\Yeti Voice"; Filename: "{app}\BlueYeti.App.exe"
