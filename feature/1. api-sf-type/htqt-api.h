#pragma once
#include <windows.h>
#include <stdint.h>

/* HTQT_API: dllexport when building DLL, dllimport when consuming */
#ifdef HTQT_DLL_EXPORTS
#define HTQT_API __declspec(dllexport)
#else
#define HTQT_API __declspec(dllimport)
#endif

#ifdef __cplusplus
extern "C" {
#endif

/* -----------------------------------------------------------------------
   Callback typedefs — all callbacks return 0 on success, non-zero on failure
   ----------------------------------------------------------------------- */

   /* ── Callback: RSA-PSS Sign ─────────────────────────────── */
   /*  Called by DLL when it needs to sign a message digest.    */
   /*  digest      : pointer to SHA-256 / SHA-512 hash bytes    */
   /*  digest_len  : byte length of digest                      */
   /*  signature   : caller-allocated buffer for signature      */
   /*  sig_len     : [in] buffer capacity, [out] bytes written  */
   /*  user_ctx    : opaque pointer forwarded from CryptoCallbacks */
   /*  returns     : HTQT_OK or negative error code           */
/* RSA-PSS-SHA256 sign: sign 'digest' (32 bytes) with caller's private key.
   Writes signature into 'signature' buffer; sets *sig_len on return. */

typedef int (*FnRsaPssSign)(
    const unsigned char *digest,    unsigned int digest_len,
    unsigned char       *signature, unsigned int *sig_len,
    void                *user_ctx);

/* ── Callback: RSA-OAEP Encrypt (wrap symmetric key) ────── */
/*  Called by DLL to RSA-encrypt (wrap) the session key      */
/*  using the recipient's PUBLIC key.                        */
/* RSA-OAEP-SHA256 encrypt 'plaintext' using public key extracted from 'cert_der'.
   Writes ciphertext into 'ciphertext_out'; sets *ciphertext_len on return. */
typedef int (*FnRsaOaepEncryptForCert)(
    const unsigned char *plaintext,      unsigned int  plaintext_len,
    const unsigned char *cert_der,       unsigned int  cert_der_len,
    unsigned char       *ciphertext_out, unsigned int *ciphertext_len,
    void                *user_ctx);

/* ── Callback: RSA-OAEP Decrypt (unwrap symmetric key) ───── */
/*  Called by DLL to recover the wrapped session key          */
/*  using the recipient's PRIVATE key on token.               */
/* RSA-OAEP-SHA256 decrypt: unwrap 'ciphertext' with caller's private key.
   Writes plaintext into 'plaintext_out'; sets *plaintext_len on return. */
typedef int (*FnRsaOaepDecrypt)(
    const unsigned char *ciphertext,   unsigned int  ciphertext_len,
    unsigned char       *plaintext_out, unsigned int *plaintext_len,
    void                *user_ctx);

/* ── Callback: RSA-PSS Verify ───────────────────────────── */
/*  Called by DLL after decryption to verify file signature.  */
/*  Uses sender's public certificate (software op).
/* RSA-PSS-SHA256 verify: verify 'sig' against 'digest' using public key from 'sender_cert_der'.
   Returns 0 if signature valid, non-zero otherwise. */
typedef int (*FnRsaPssVerify)(
    const unsigned char *digest,         unsigned int digest_len,
    const unsigned char *sig,            unsigned int sig_len,
    const unsigned char *sender_cert_der, unsigned int sender_cert_der_len,
    void                *user_ctx);

/* Progress callback: called after each (file_idx, recip_idx) pair completes.
   Return non-zero to cancel remaining operations. */
typedef int (*FnProgressCallback)(
    uint32_t file_idx,   uint32_t recip_idx,
    uint32_t file_total, uint32_t recip_total,
    int      status,     void *user_ctx);

/* -----------------------------------------------------------------------
   CryptoCallbacksV2 -- caller populates and passes to enc/dec functions
   ----------------------------------------------------------------------- */
typedef struct {
    FnRsaPssSign            sign_fn;          /* RSA-PSS sign with private key on token */
    FnRsaOaepEncryptForCert rsa_enc_cert_fn;  /* RSA-OAEP encrypt (key-wrap)            */
    FnRsaOaepDecrypt        rsa_dec_fn;        /* RSA-OAEP encrypt (key-wrap)            */
    FnRsaPssVerify          verify_fn;         /* RSA-PSS verify (public key)  */
    FnProgressCallback      progress_fn;       /* Optional; return non-0 to cancel */
    void                   *user_ctx;          /* Passed as-is to all callbacks */
    const unsigned char    *own_cert_der;      /* Required for SF v1 compat decrypt */
    unsigned int            own_cert_der_len;  /* 0 = skip SF v1 fingerprint matching */
    void                   *reserved[3];       /* Must be NULL */
} CryptoCallbacksV2;

/* -----------------------------------------------------------------------
   Batch encrypt input structs
   ----------------------------------------------------------------------- */
typedef struct {
    const char *input_path;  /* UTF-8 path to plaintext file */
    const char *file_id;     /* Used in output filename: {file_id}-{recipient_id}.sf */
} FileEntry;

typedef struct {
    const char *cert_path;    /* UTF-8 path to recipient DER or PEM certificate */
    const char *recipient_id; /* Stored in SF v2 RecipientBlock, used in output filename */
} RecipientEntry;

typedef struct {
    const FileEntry      *files;
    uint32_t              file_count;
    const RecipientEntry *recipients;
    uint32_t              recipient_count;
    const char           *output_dir;    /* UTF-8 path to output directory */
    uint32_t              flags;         /* HTQT_BATCH_* flags */
    void                 *reserved[2];   /* Must be NULL */
} BatchEncryptParams;


/* -----------------------------------------------------------------------
   Batch result: one entry per (file, recipient) pair
   ----------------------------------------------------------------------- */
typedef struct {
    uint32_t file_index;
    uint32_t recipient_index;
    int      status;             /* HTQT_OK or HTQT_ERR_* */
    char     output_path[512];   /* UTF-8 path of output .sf file */
    char     error_detail[256];  /* Human-readable error description */
} BatchResult;

/* -----------------------------------------------------------------------
   Batch flags
   ----------------------------------------------------------------------- */
#define HTQT_BATCH_CONTINUE_ON_ERROR  0x01u  /* Continue processing after per-item failures */
#define HTQT_BATCH_OVERWRITE_OUTPUT   0x02u  /* Overwrite existing output files */

/* -----------------------------------------------------------------------
    Exported functions
   ----------------------------------------------------------------------- */

/* Batch encrypt M files -> SF v1 format.
   Produces one SF v1 output file per input file with all N recipient blocks embedded.
   results[] must have capacity >= file_count (NOT file_count * recipient_count).
   Output files: {output_dir}/{file_id}.sf1
   Returns HTQT_OK, HTQT_ERR_PARTIAL, or HTQT_ERR_* on total failure. */

HTQT_API int encHTQT_sf_multi(
    const BatchEncryptParams *params,
    const CryptoCallbacksV2  *cbs,
    BatchResult              *results,
    char                     *error_msg,
    int                       error_len);


/* Decrypt a single SF v1 (.sf1) file.
   sf1_path      - path to input .sf1 file
   output_dir    - directory to write decrypted output (filename taken from orig_name in header)
   cbs           - crypto callbacks; own_cert_der required for fingerprint matching
   flags         - HTQT_BATCH_* flags (e.g. HTQT_BATCH_OVERWRITE_OUTPUT)
   out_path_buf  - receives the output file path on success
   out_path_buf_len - capacity of out_path_buf
   err_buf       - receives a human-readable error message on failure
   err_len       - capacity of err_buf
   Returns HTQT_OK or HTQT_ERR_* on failure. */
HTQT_API int decrypt_one_sfv1(
    const char              *sf1_path,
    const char              *output_dir,
    const CryptoCallbacksV2 *cbs,
    uint32_t                 flags,
    char                    *out_path_buf,
    int                      out_path_buf_len,
    char                    *err_buf,
    int                      err_len);

/* Returns the last error or warning code.
   Returns HTQT_OK (0) if no error occurred. */
HTQT_API int HTQT_GetError(void);

#ifdef __cplusplus
}
#endif
