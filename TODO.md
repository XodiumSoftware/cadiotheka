# Bugs / Fixes

A0a. [ ] Fix tag/platform overflow on project cards where the `+<int>` on hover is not showing the remaining tags/platforms.
A0b. [x] Fix the 3D viewer fullscreen showing an empty project modal instead of the 3D viewer only.
A0c. [x] 3D-viewer controls do not work (pan/move)
A0d. [ ] Profile button in the header doesnt show tooltip even when defined.

# `three-d` Viewer Enhancement TODO

## Rendering Quality

A1. [x] Add ambient fill light + soft shadow mapping for the directional light.
A2. [ ] Load an HDR skybox for image-based lighting (IBL) on `PhysicalMaterial`.
A3. [ ] Implement MSAA via an intermediate multi-sampled render target.
A4. [ ] Add exposure/tone-mapping controls.

## Model Display

A5. [ ] Exploded-view / ghost-mode toggle for inspecting IFC assemblies.
A6. [ ] Interactive section / cut-plane slider.
A7. [ ] Wireframe / edges overlay on top of shaded geometry.
A8. [ ] Back-face culling toggle.
A9. [ ] Orthographic camera mode alongside the current perspective view.

## Interaction

A10. [ ] Raycast hover / picking to highlight an element and show its metadata.
A11. [ ] Point-to-point measurement tool with on-canvas distance label.
A12. [ ] Auto-rotate / turntable animation mode.
A13. [ ] View-cube or preset view buttons (front, top, isometric).

## Data & Media

A14. [ ] GLB animation playback and skinning support.
A15. [ ] Extra model stats: surface area, bounding-box dimensions.
A16. [ ] Multi-model comparison / diff overlay with per-model tinting.
A18. [ ] Project thumbnail generation from the viewer for use in cards.

## App Integration

A19. [x] IFC upload-to-GLB conversion pipeline status UI. After upload, the backend eagerly converts and caches the GLB via `POST /data/projects/:id/glb`; the Versions tab shows `Converting`/`Ready`/`NoGeometry`/`Failed` states.
A20. [ ] Lightweight static preview on project cards instead of full 3D.
A21. [x] Persist per-project viewer settings in local storage alongside `ViewState`.
A22. [ ] Add headless test coverage for the 3D viewer (scene/camera math and control event handling) so orbit/pan/zoom behavior is verified without a browser.

# Other stuff

C1a. [x] Move the plaforms and tags to the database.
C1b. [ ] Adjust profile dropdown for admin's to have a AdminModal where the admin can remove/create/rename platforms/tags.
C2. [ ] Rename the subprojects to remove the prefix.
