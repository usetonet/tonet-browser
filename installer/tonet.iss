; Instalador EXE con Inno Setup 6 (estilo instaladores tipo Firefox/Chrome offline).
; Requisitos: Inno Setup 6 + cargo build --release
; Compilar: ISCC.exe installer\tonet.iss (desde la raíz del repo o ajustar rutas)

#define MyAppName "Tonet"
#define MyAppVersion "0.1.1"
#define MyAppPublisher "usetonet.com"
#define MyAppURL "https://usetonet.com"
#define MyAppExeName "tonet.exe"

[Setup]
AppId={{E7C8F9A0-1234-5678-9ABC-DEF012345678}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
OutputDir=..\dist
OutputBaseFilename=Tonet-Setup-{#MyAppVersion}-x64
UninstallDisplayIcon={app}\{#MyAppExeName}
; Same icon as embedded in tonet.exe (crates/tonet/windows/app.ico)
SetupIconFile=..\crates\tonet\windows\app.ico

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; Servo/surfman loads ANGLE from the exe directory (see crates/tonet/build.rs).
Source: "..\target\release\libEGL.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\libGLESv2.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Comment: "Navegador Tonet"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
