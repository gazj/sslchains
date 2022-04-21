# ⛓ sslchains

A tool to identify related SSL keys, CSRs, and certificates.

## Usage

#### Default Display Mode

Run with any number of path arguments to define the working file set. If no arguments are provided, the current working directory (`$PWD`) is used.

```
% sslchains samples/*pem
example.org
  * Key: samples/self_signed.pem
  * CSR: samples/self_signed.pem
  * Certificates:
    - samples/self_signed.pem (self-signed)
```

Reference separate key, CSR, and certificate files (including signing certificates) to generate more complete output.

```
% sslchains samples/ca* samples/intermediate_ca.crt
example.com
  * Key: samples/ca_signed.key
  * CSR: samples/ca_signed.csr
  * Certificates:
    - samples/ca_signed.crt
      > samples/intermediate_ca.crt
```

#### OneLine Display Mode

Use with the `-l` option to display each chain on a single line (`-L` to suppress the header row).

In OneLine display mode, each certificate in the chain is separated by a pipe (`|`) symbol.

_Tip: Add `|column -t` for more readable output._

```
% sslchains -l samples/ca* samples/intermediate_ca.crt | column -t
name         key                    request                certificate_chain
example.com  samples/ca_signed.key  samples/ca_signed.csr  samples/ca_signed.crt|samples/intermediate_ca.crt
```

## Contributing

Pull requests are welcome.

## License

[GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)