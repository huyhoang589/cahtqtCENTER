# Brainstorm: License Change v1

**Date:** 2026-04-07
**Branch:** feature/licenseGen
**Spec:** `feature/3. licenseGen/3. license_change_v1.txt`

## Problem Statement
Three changes to license gen feature:
1. UI: Split preview into 2 cards (credential left, payload right), unmask token serial, black text
2. Path: Output to `{output_data_dir}\SF\LICENSE\{User_name}\`
3. DB: Store license blob, add export & delete buttons

## Decisions

| Question | Decision |
|---|---|
| Preview card content | Both cards: left=credential fields, right=payload fields, separated by divider |
| DB storage | New `license_blob TEXT` column on `license_audit` table |
| Delete behavior | Hard delete (DB row + disk file) |
| Export filename | Keep `{userName}-license.dat` |

## Design

### UI — Split Preview Cards

| Left: "Credential Preview" | Right: "License Preview" |
|---|---|
| User Name | Product |
| Token Serial (full, unmasked) | Issued By |
| CPU ID | Issued At |
| Board Serial | Machine FP |
| Registered At | Expires At |
| | Token Serial |

- All text inside cards: black
- Right card populated only after generation

### Path Change
```
Before: {output_data_dir}\LICENSE\{userName}-license.dat
After:  {output_data_dir}\SF\LICENSE\{User_name}\{userName}-license.dat
```

### DB Migration (006)
```sql
ALTER TABLE license_audit ADD COLUMN license_blob TEXT;
```

### New Commands
- `export_license(audit_id)` — DB blob → file at SF/LICENSE/{User_name}/
- `delete_license(audit_id)` — Delete DB row + disk file

### Frontend Changes
- Export/Delete buttons per audit row
- Confirmation dialog for delete
- Split preview layout

## Risks
- Existing rows have NULL blob → export disabled for those
- Old files stay at old path; only new generations use new path
- License blobs small (~1-2KB), no SQLite concern
