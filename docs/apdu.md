
# APDUs

The messaging format of the app is compatible with the [APDU protocol](https://developers.ledger.com/docs/nano-app/application-structure/#apdu-interpretation-loop).

All commands use `CLA = 0x00`.
The `P1` and `P2` fields are reserved for future use and must be set to `0` in all messages.

| CLA | INS | COMMAND NAME     | DESCRIPTION                                                    |
|-----|-----|------------------|----------------------------------------------------------------|
| 00  | 00  | GET_VERSION      | Gets the app version in machine readable format (bytes)        |
| 00  | 02  | GET_PUBKEY       | Gets the Public Key and Address for a BIP32 path               |
| 00  | 03  | SIGN_JSON_TX     | Sign a Transaction specified in JSON                           |
| 00  | 04  | SIGN_TX_HASH     | Sign a Transaction Hash (requires Blind Signing to be enabled) |
| 00  | 10  | MAKE_TRANSFER_TX | Build a transfer transaction and sign it                       |
| 00  | FE  | GET_VERSION_STR  | Gets the app version in string                                 |
| 00  | FF  | QUIT_APP         | Quits the app                                                  |


### GET_VERSION

Returns the version of the app currently running on the Ledger in machine readable format (bytes)

#### Encoding

**Command**

| *CLA* | *INS* |
|-------|-------|
| 00    | 00    |

**Output data**

| Length       | Description     |
|--------------|-----------------|
| `1`          | Major version   |
| `1`          | Minor version   |
| `1`          | Patch version   |
| `<variable>` | Name of the app |

### GET_PUBKEY

Returns the public key and the address for the given derivation path.

#### Encoding

**Command**

| *CLA* | *INS* |
|-------|-------|
| 00    | 02    |

**Input data**

| Length | Name              | Description                         |
|--------|-------------------|-------------------------------------|
| `1`    | `n`               | Number of derivation steps          |
| `4`    | `bip32_path[0]`   | First derivation step (big endian)  |
| `4`    | `bip32_path[1]`   | Second derivation step (big endian) |
|        | ...               |                                     |
| `4`    | `bip32_path[n-1]` | `n`-th derivation step (big endian) |

**Output data**

| Length       | Description                  |
|--------------|------------------------------|
| `1`          | The length of the public key |
| `<variable>` | Public key                   |
| `1`          | The length of the address    |
| `<variable>` | Address                      |

### SIGN_JSON_TX

Sign a Transaction in JSON format encoded in hexadecimal string (utf8), using the key for the given derivation path

#### Encoding

**Command**

| *CLA* | *INS* |
|-------|-------|
| 00    | 03    |

**Input data**

| Length    | Name              | Description                         |
|-----------|-------------------|-------------------------------------|
| `4`       | `tx_size`         | Size of transaction                 |
| `tx_size` | `tx`              | Transaction in hexadecimal string   |
| `1`       | `n`               | Number of derivation steps          |
| `4`       | `bip32_path[0]`   | First derivation step (big endian)  |
| `4`       | `bip32_path[1]`   | Second derivation step (big endian) |
|           | ...               |                                     |
| `4`       | `bip32_path[n-1]` | `n`-th derivation step (big endian) |

**Output data**

| Length       | Description     |
|--------------|-----------------|
| `<variable>` | Signature bytes |

### SIGN_TX_HASH

Sign a Transaction hash, using the key for the given derivation path.
This APDU requires the Blind Signing to be enabled on the Ledger app.

#### Encoding

**Command**

| *CLA* | *INS* |
|-------|-------|
| 00    | 04    |

**Input data**

| Length | Name              | Description                         |
|--------|-------------------|-------------------------------------|
| `32`   | `tx_hash`         | Transaction hash                    |
| `1`    | `n`               | Number of derivation steps          |
| `4`    | `bip32_path[0]`   | First derivation step (big endian)  |
| `4`    | `bip32_path[1]`   | Second derivation step (big endian) |
|        | ...               |                                     |
| `4`    | `bip32_path[n-1]` | `n`-th derivation step (big endian) |

**Output data**

| Length       | Description     |
|--------------|-----------------|
| `<variable>` | Signature bytes |

### MAKE_TRANSFER_TX

Builds a transfer transaction using the input data, and provides a signature for it.

The transaction `cmd` JSON string is constructed based on the following templates.
Here the `$PUBKEY` is derived from the given `bip32_path`.
All other parameters  are specified as utf8 encoding string in the input data.

1. Transfer

```
"{\"networkId\":\"$NETWORK\",\"payload\":{\"exec\":{\"data\":{},\"code\":\"(coin.transfer \\\"k:$PUBKEY\\\" \\\"k:$RECIPIENT\\\" $AMOUNT)\"}},\"signers\":[{\"pubKey\":\"$PUBKEY\",\"clist\":[{\"args\":[\"k:$PUBKEY\",\"k:$RECIPIENT\",$AMOUNT],\"name\":\"coin.TRANSFER\"},{\"args\":[],\"name\":\"coin.GAS\"}]}],\"meta\":{\"creationTime\":$CREATION_TIME,\"ttl\":$TTL,\"gasLimit\":$GAS_LIMIT,\"chainId\":\"$CHAIN_ID\",\"gasPrice\":$GAS_PRICE,\"sender\":\"k:$PUBKEY\"},\"nonce\":\"$NONCE\"}"
```

2. Transfer Create

```
"{\"networkId\":\"$NETWORK\",\"payload\":{\"exec\":{\"data\":{\"ks\":{\"pred\":\"keys-all\",\"keys\":[\"$PUBKEY\"]}},\"code\":\"(coin.transfer-create \\\"k:$PUBKEY\\\" \\\"k:$RECIPIENT\\\" (read-keyset \\\"ks\\\") $AMOUNT)\"}},\"signers\":[{\"pubKey\":\"$PUBKEY\",\"clist\":[{\"args\":[\"k:$PUBKEY\",\"k:$RECIPIENT\",$AMOUNT],\"name\":\"coin.TRANSFER\"},{\"args\":[],\"name\":\"coin.GAS\"}]}],\"meta\":{\"creationTime\":$CREATION_TIME,\"ttl\":$TTL,\"gasLimit\":$GAS_LIMIT,\"chainId\":\"$CHAIN_ID\",\"gasPrice\":$GAS_PRICE,\"sender\":\"k:$PUBKEY\"},\"nonce\":\"$NONCE\"}"
```

3. Cross-Chain Transfer

```
"{\"networkId\":\"$NETWORK\",\"payload\":{\"exec\":{\"data\":{\"ks\":{\"pred\":\"keys-all\",\"keys\":[\"$PUBKEY\"]}},\"code\":\"(coin.transfer-crosschain \\\"k:$PUBKEY\\\" \\\"k:$RECIPIENT\\\" (read-keyset \\\"ks\\\") \\\"$RECIPIENT_CHAIN\\\" $AMOUNT)\"}},\"signers\":[{\"pubKey\":\"$PUBKEY\",\"clist\":[{\"args\":[\"k:$PUBKEY\",\"k:$RECIPIENT\",$AMOUNT,\"$RECIPIENT_CHAIN\"],\"name\":\"coin.TRANSFER_XCHAIN\"},{\"args\":[],\"name\":\"coin.GAS\"}]}],\"meta\":{\"creationTime\":$CREATION_TIME,\"ttl\":$TTL,\"gasLimit\":$GAS_LIMIT,\"chainId\":\"$CHAIN_ID\",\"gasPrice\":$GAS_PRICE,\"sender\":\"k:$PUBKEY\"},\"nonce\":\"$NONCE\"}"
```

#### Encoding

**Command**

| *CLA* | *INS* |
|-------|-------|
| 00    | 10    |

**Input data**

| Length       | Name                  | Description                              |
|--------------|-----------------------|------------------------------------------|
| `1`          | `n`                   | Number of derivation steps               |
| `4`          | `bip32_path[0]`       | First derivation step (big endian)       |
| `4`          | `bip32_path[1]`       | Second derivation step (big endian)      |
|              | ...                   |                                          |
| `4`          | `bip32_path[n-1]`     | `n`-th derivation step (big endian)      |
| `1`          | `tx_type`             | Type of transaction                      |
| `1`          | `recipient_len`       | Recipient pubkey length (should be 64)   |
| `64`         | `recipient`           | Recipient pubkey (in hex)                |
| `1`          | `recipient_chain_len` | Recipient Chain Id length (<= 2)         |
| `<variable>` | `recipient_chain`     | Recipient Chain Id (utf8 encoded string) |
| `1`          | `network_len`         | Network length (<= 20)                   |
| `<variable>` | `network`             | Network (utf8 encoded string)            |
| `1`          | `amount_len`          | Amount length (<= 32)                    |
| `<variable>` | `amount`              | Amount (utf8 encoded string)             |
| `1`          | `namespace_len`       | Namespace length (<= 16)                 |
| `<variable>` | `namespace`           | Namespace (utf8 encoded string)          |
| `1`          | `module_len`          | Module name length (<= 32)               |
| `<variable>` | `module`              | Module name (utf8 encoded string)        |
| `1`          | `gas_price_len`       | Gas price length (<= 20)                 |
| `<variable>` | `gas_price`           | Gas price (utf8 encoded string)          |
| `1`          | `gas_limit_len`       | Gas limit length (<= 10)                 |
| `<variable>` | `gas_limit`           | Gas limit (utf8 encoded string)          |
| `1`          | `creation_time_len`   | Creation time length (<= 12)             |
| `<variable>` | `creation_time`       | Creation time (utf8 encoded string)      |
| `1`          | `chain_id_len`        | Chain id length (<= 2)                   |
| `<variable>` | `chain_id`            | Chain id (utf8 encoded string)           |
| `1`          | `nonce_len`           | Nonce length (<= 32)                     |
| `<variable>` | `nonce`               | Nonce (utf8 encoded string)              |
| `1`          | `ttl_len`             | TTL length (<= 20)                       |
| `<variable>` | `ttl`                 | TTL (utf8 encoded string)                |

| `tx_type` | Description          |
|-----------|----------------------|
| 0         | Transfer             |
| 1         | Transfer Create      |
| 2         | Cross-chain Transfer |

**Output data**

| Length | Description                 |
|--------|-----------------------------|
| `64`   | Signature bytes             |
| `32`   | Public key used for signing |

### GET_VERSION_STR

Returns the name of the app currently running on the Ledger, including its version, like 'Kadena 0.1.2'

#### Encoding

**Command**

| *CLA* | *INS* |
|-------|-------|
| 00    | FF    |

**Output data**

| Length       | Description               |
|--------------|---------------------------|
| `<variable>` | Name of the app + version |

## Status Words

| SW     | SW name                       | Description                                                |
|--------|-------------------------------|------------------------------------------------------------|
| 0x6808 | `SW_NOT_SUPPORTED`            | `INS` is disabled  (Blind Signing)                         |
| 0x6982 | `SW_NOTHING_RECEIVED`         | No input was received by the app                           |
| 0x6D00 | `SW_ERROR`                    | Error has occured due to bad input or user rejectected     |
| 0x6E00 | `SW_CLA_OR_INS_NOT_SUPPORTED` | No command exists for the `CLA` and `INS`                  |
| 0x9000 | `SW_OK`                       | Success, or continue if more input from client is expected |
