{ pkgs ? import <nixpkgs> { config = {allowUnfree = true;}; } }: pkgs.mkShell {
    name = "env";
    nativeBuildInputs = with pkgs; [
        pkg-config
    ];
    buildInputs = with pkgs; [
        # needed for sound
        alsaLib

        # needed to connect to wayland windowing api
        wayland
        wayland-protocols
        egl-wayland

        # needed for keyboard
        libxkbcommon
        
        # needed for egl/openGL
        libGLU 

        # needed for font rendering
        freetype
    ];
}
