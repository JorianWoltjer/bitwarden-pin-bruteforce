# Bitwarden PIN Bruteforce

**Brute force any Bitwarden PIN from storage or in-memory**

> [!WARNING]  
> This README is very incomplete still, I promise I'll make a proper one soon

Default settings:

```bash
bitwarden-pin -e "2.P6TpPPpMf5zkHUfTplnocw==|KZ7/pR8ft+LwcjfXs2ym9hmxE7DLIeA9Kl+IPwTVCwLmbpkFtYKPWvK53DEDDrVUeYvz/rPcl3MEH3wXl200HCsV5ZbGLGVU4bha5Aw20fk=|+Y46Za3Oo63XRbvqLFz5cVuvbqMvBqopD16+8HV83mk=" -m "tenire3448@fashlend.com"
```

Set custom iterations to old version, 6x faster to crack:

```bash
bitwarden-pin -e "..." -m "tenire3448@fashlend.com" pbkdf2 -i 100000
```

Set algorithm to newer argon2:

```bash
bitwarden-pin -e "2.FA4aPsq/5jKajc8tGqYKaQ==|CO/t9f1EQ4O5LL6O1anBAd1/4Hb+l4I32UMlW+3O7CoxTRXlEuLK5xvDCFmeRCYmylt206B22roFXycaRG3Z9fnN1aVVbBJ59qfCDEGusHw=|vmWmAb9kfqPPljRNhDMe+fDlwwat8XN5BZSsMAH8p8w=" -m "tenire3448@fashlend.com" argon2
```

Set custom kdfConfig values for argon2:

```bash
bitwarden-pin -e "2.FA4aPsq/5jKajc8tGqYKaQ==|CO/t9f1EQ4O5LL6O1anBAd1/4Hb+l4I32UMlW+3O7CoxTRXlEuLK5xvDCFmeRCYmylt206B22roFXycaRG3Z9fnN1aVVbBJ59qfCDEGusHw=|vmWmAb9kfqPPljRNhDMe+fDlwwat8XN5BZSsMAH8p8w=" -m "tenire3448@fashlend.com" argon2 -i 3 -m 64 -p 4
```

Tip: use `-p` to set number of digits in PIN. Commonly this is 4, but it can be anything the user inputs.
