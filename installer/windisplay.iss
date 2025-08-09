#define MyAppVersion "0.1.10"
#define SourceDir ".."

[Setup]
AppId={{9C7B2E3A-9F2F-4B7E-9C8B-1B1C2D3E4F5A}}
AppName=WinDisplay
AppVersion={#MyAppVersion}
AppPublisher=zpix1
AppPublisherURL=https://github.com/zpix1/windisplay
DefaultDirName={autopf}\WinDisplay
DefaultGroupName=WinDisplay
DisableProgramGroupPage=yes
OutputBaseFilename=WinDisplay-Setup-{#MyAppVersion}
OutputDir=Output
Compression=lzma
SolidCompression=yes
WizardStyle=modern
SetupIconFile={#SourceDir}\\windisplay\\assets\\app.ico

[Files]
Source: "{#SourceDir}\\dist\\WinDisplay.exe"; DestDir: "{app}"; Flags: ignoreversion

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked

[Icons]
Name: "{autoprograms}\\WinDisplay"; Filename: "{app}\\WinDisplay.exe"; IconFilename: "{app}\\WinDisplay.exe"
Name: "{autodesktop}\\WinDisplay"; Filename: "{app}\\WinDisplay.exe"; Tasks: desktopicon; IconFilename: "{app}\\WinDisplay.exe"


