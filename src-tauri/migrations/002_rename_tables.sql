ALTER TABLE groups RENAME TO partners;
ALTER TABLE recipients RENAME TO partner_members;
ALTER TABLE enc_logs RENAME COLUMN recipient_id TO partner_member_id;
ALTER TABLE partner_members RENAME COLUMN group_id TO partner_id
