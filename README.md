# ⛓ sslchains

A tool to identify related SSL keys, CSRs, and certificates.

## Usage

#### Show usage

Use with the `-h` option to show usage information.

#### Process hidden files / directories

Use with the `-H` option to include hidden files and directories.

#### Default display mode

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

#### Single line display mode

Use with the `-l` option to display each chain on a single line (`-L` to suppress the header row).

In OneLine display mode, each certificate in the chain is separated by a pipe (`|`) symbol.

_Tip: Add `|column -t` for more readable output._

```
% sslchains -l samples/ca* samples/intermediate_ca.crt | column -t
name         key                    request                certificate_chain
example.com  samples/ca_signed.key  samples/ca_signed.csr  samples/ca_signed.crt|samples/intermediate_ca.crt
```

#### Process arguments recursively

Use with the `-r` option to process arguments recursively.

#### Follow symbolic links

Use with the `-S` option to follow symlinks.

#### Process an unlimited number of files

Use with the `-U` option to process an unlimited number of files, rather than exiting after a set default limit.

#### Cross filesystem boundaries

Use with the `-X` option to cross filesystem boundaries.

## Contributing

Pull requests are welcome.

## License

[GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)
