# LVGL Pro Viewer Mockups

This folder is a small LVGL Pro project you can open directly in the editor or in the online viewer.

- `project.xml` defines two targets:
  - `whisplay_portrait` for `screens/hub.xml`
  - `standard_landscape` for `screens/main_menu.xml`
- `globals.xml` carries the shared YoyoPod palette pulled from `yoyopy/ui/screens/theme.py`.
- `translations.xml` is intentionally minimal so the editor/codegen pipeline has a valid translations file to load.
- The screen XML files are viewer-friendly recreations of:
  - `yoyopy/ui/screens/navigation/hub.py`
  - `yoyopy/ui/screens/navigation/menu.py`

The screen files are intentionally self-contained now, so they can also be pasted into the LVGL editor as single files without relying on `globals.xml`.

If you want to iterate fast, start by changing colors, radii, positions, and copy inside the two screen files.
