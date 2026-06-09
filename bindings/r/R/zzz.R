.onLoad <- function(libname, pkgname) {
  # On Windows the package's wickra.dll depends on the bundled C ABI
  # wickra_abi.dll; the loader searches PATH for it, so prepend the package's own
  # libs directory. On Linux/macOS the rpath baked at build time locates the
  # shared library, so no PATH change is needed.
  if (.Platform$OS.type == "windows") {
    libs <- system.file(paste0("libs", .Platform$r_arch),
                        package = pkgname, lib.loc = libname)
    if (nzchar(libs)) {
      Sys.setenv(PATH = paste(libs, Sys.getenv("PATH"), sep = .Platform$path.sep))
    }
  }
  library.dynam("wickra", pkgname, libname)
}

.onUnload <- function(libpath) {
  library.dynam.unload("wickra", libpath)
}
