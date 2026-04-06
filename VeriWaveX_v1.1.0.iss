[Setup]
AppName=VeriWaveX
AppVersion=1.1.0
AppPublisher=Aritrash Sarkar
DefaultDirName={autopf}\VeriWaveX
DefaultGroupName=VeriWaveX
UninstallDisplayIcon={app}\veriwavex.exe
Compression=lzma2/max
SolidCompression=yes
; Update the output name for the new version
OutputBaseFilename=VeriWaveX_v1.1.0_Setup
SetupIconFile=assets\logo.ico
OutputDir=dist

[Files]
; The new v1.1.0 binary
Source: "target\release\veriwavex.exe"; DestDir: "{app}"; Flags: ignoreversion
; Include the toolchains and assets
Source: "vendor\*"; DestDir: "{app}\vendor"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{autoprograms}\VeriWaveX"; Filename: "{app}\veriwavex.exe"; IconFilename: "{app}\assets\logo.ico"
Name: "{autodesktop}\VeriWaveX"; Filename: "{app}\veriwavex.exe"; IconFilename: "{app}\assets\logo.ico"

[Run]
Filename: "{app}\veriwavex.exe"; Description: "{cm:LaunchProgram,VeriWaveX}"; Flags: nowait postinstall skipifsilent