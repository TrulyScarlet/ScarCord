fn main() {
    let mut windows_attributes = tauri_build::WindowsAttributes::new();
    windows_attributes = windows_attributes.app_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="0.1.0.0" name="TrulyScarlet.ScarCord" type="win32" />
  <description>ScarCord - Lightweight Discord Desktop Client</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#);

    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(windows_attributes)
    ).expect("failed to run tauri-build");
}
