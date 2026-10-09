#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef StageDir
  #define StageDir "..\..\dist\stage"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif
[Setup]
AppId={{2F9B4C29-1D43-4A70-86AB-24ED3088ACD7}
AppName=Starframe FITS
AppVersion={#AppVersion}
AppPublisher=Aoi-Morio and contributors
AppPublisherURL=https://github.com/Aoi-Morio/starframe-fits
AppSupportURL=https://github.com/Aoi-Morio/starframe-fits/issues
AppUpdatesURL=https://github.com/Aoi-Morio/starframe-fits/releases/latest
DefaultDirName={localappdata}\Programs\Starframe FITS
DefaultGroupName=Starframe FITS
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
DisableProgramGroupPage=yes
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
OutputDir={#OutputDir}
OutputBaseFilename=starframe-fits-{#AppVersion}-windows-x64-setup
SetupIconFile=..\..\assets\app-icon\photocraft.ico
UninstallDisplayIcon={app}\StarframeFITS.exe
CloseApplications=yes
RestartApplications=no
LicenseFile=..\..\LICENSE-MIT
[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
[Files]
Source: "{#StageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs; Excludes: "portable.txt,StarframeData\*,PhotoCraftData\*,*.pdb"
[Icons]
Name: "{userprograms}\Starframe FITS"; Filename: "{app}\StarframeFITS.exe"
Name: "{userdesktop}\Starframe FITS"; Filename: "{app}\StarframeFITS.exe"; Tasks: desktopicon
[Registry]
Root: HKCU; Subkey: "Software\Classes\Applications\StarframeFITS.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "Starframe FITS"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\StarframeFITS.exe\shell\open\command"; ValueType: string; ValueData: """{app}\StarframeFITS.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\Applications\StarframeFITS.exe\SupportedTypes"; ValueType: string; ValueName: ".fits"
Root: HKCU; Subkey: "Software\Classes\Applications\StarframeFITS.exe\SupportedTypes"; ValueType: string; ValueName: ".fit"
Root: HKCU; Subkey: "Software\Classes\Applications\StarframeFITS.exe\SupportedTypes"; ValueType: string; ValueName: ".fts"
Root: HKCU; Subkey: "Software\Classes\StarframeFITS.Image"; ValueType: string; ValueData: "Starframe FITS image"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\StarframeFITS.Image\DefaultIcon"; ValueType: string; ValueData: "{app}\StarframeFITS.exe,0"
Root: HKCU; Subkey: "Software\Classes\StarframeFITS.Image\shell\open\command"; ValueType: string; ValueData: """{app}\StarframeFITS.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\.fits\OpenWithProgids"; ValueType: string; ValueName: "StarframeFITS.Image"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.fit\OpenWithProgids"; ValueType: string; ValueName: "StarframeFITS.Image"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.fts\OpenWithProgids"; ValueType: string; ValueName: "StarframeFITS.Image"; Flags: uninsdeletevalue
[Run]
Filename: "{app}\StarframeFITS.exe"; Description: "{cm:LaunchProgram,Starframe FITS}"; Flags: nowait postinstall skipifsilent
