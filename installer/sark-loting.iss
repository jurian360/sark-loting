[Setup]
AppName=SARK Loting
AppVersion=0.1.0
AppPublisher=SARK
DefaultDirName={autopf}\SARK Loting
DefaultGroupName=SARK Loting
OutputBaseFilename=SARK-Loting-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequiredOverridesAllowed=dialog
UninstallDisplayIcon={app}\sark-loting.exe

[Languages]
Name: "dutch"; MessagesFile: "compiler:Languages\Dutch.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\target\release\sark-loting.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\SARK Loting"; Filename: "{app}\sark-loting.exe"
Name: "{autodesktop}\SARK Loting"; Filename: "{app}\sark-loting.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\sark-loting.exe"; Description: "{cm:LaunchProgram,SARK Loting}"; Flags: nowait postinstall skipifsilent
