[Setup]
AppName=VeriWaveX
AppVersion=1.0-beta
DefaultDirName={autopf}\VeriWaveX
DefaultGroupName=VeriWaveX
UninstallDisplayIcon={app}\VeriWaveX.exe
Compression=lzma2/max
SolidCompression=yes
OutputDir=dist
OutputBaseFilename=VeriWaveX_Setup
SetupIconFile=assets\logo.ico
PrivilegesRequired=lowest

[Files]
; The main application
Source: "target\release\veriwavex-beta.exe"; DestDir: "{app}"; DestName: "VeriWaveX.exe"; Flags: ignoreversion
; The entire vendor folder with all its subfolders
Source: "vendor\*"; DestDir: "{app}\vendor"; Flags: ignoreversion recursesubdirs createallsubdirs
; The assets (if needed at runtime, though we embedded them, it's safe to keep)
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{group}\VeriWaveX"; Filename: "{app}\VeriWaveX.exe"; IconFilename: "{app}\assets\logo.ico"
Name: "{autodesktop}\VeriWaveX"; Filename: "{app}\VeriWaveX.exe"; IconFilename: "{app}\assets\logo.ico"

[Run]
Filename: "{app}\VeriWaveX.exe"; Description: "Launch VeriWaveX"; Flags: nowait postinstall skipifsilent