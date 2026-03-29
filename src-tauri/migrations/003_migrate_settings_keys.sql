-- Migration v3: Rename settings keys to match eToken module naming
UPDATE settings SET key = 'pkcs11_library_path' WHERE key = 'pkcs11_lib_path';
UPDATE settings SET key = 'sender_cn' WHERE key = 'sender_cert_cn';
