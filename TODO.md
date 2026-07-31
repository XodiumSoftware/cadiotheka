# `three-d` Viewer Enhancement TODO

## Rendering Quality

1. [x] Add ambient fill light + soft shadow mapping for the directional light.
2. [ ] Load an HDR skybox for image-based lighting (IBL) on `PhysicalMaterial`.
3. [ ] Implement MSAA via an intermediate multi-sampled render target.
4. [ ] Add exposure/tone-mapping controls and HDR screenshot output.

## Model Display

5. [ ] Exploded-view / ghost-mode toggle for inspecting IFC assemblies.
6. [ ] Interactive section / cut-plane slider.
7. [ ] Wireframe / edges overlay on top of shaded geometry.
8. [ ] Back-face culling toggle.
9. [ ] Orthographic camera mode alongside the current perspective view.

## Interaction

10. [ ] Raycast hover / picking to highlight an element and show its metadata.
11. [ ] Point-to-point measurement tool with on-canvas distance label.
12. [ ] Auto-rotate / turntable animation mode.
13. [ ] View-cube or preset view buttons (front, top, isometric).

## Data & Media

14. [ ] GLB animation playback and skinning support.
15. [ ] Extra model stats: surface area, bounding-box dimensions.
16. [ ] Multi-model comparison / diff overlay with per-model tinting.
17. [ ] 2D drawing / SVG export from the current camera view.
18. [ ] Project thumbnail generation from the viewer for use in cards.

## App Integration

19. [ ] IFC upload-to-GLB conversion pipeline status UI.
20. [ ] Lightweight static preview on project cards instead of full 3D.
21. [ ] Persist per-project viewer settings in local storage alongside `ViewState`.
