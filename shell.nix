{ pkgs ? import <nixpkgs> { config = {allowUnfree = true;}; } }: pkgs.mkShell {
    name = "env";
    nativeBuildInputs = with pkgs; [
        pkg-config
    ];
    buildInputs = with pkgs; [
        libudev

        # needed for sound
        alsaLib

        # needed to connect to wayland windowing api
        wayland
        wayland-protocols

        # needed for keyboard
        libxkbcommon
        
        # needed to connect to vulkan graphics api
        vulkan-tools
        vulkan-headers
        vulkan-loader
        vulkan-validation-layers
    ];
}
