# Update openssl.cnf and v3.ext, then run this script to create a new self-signed certificate.
openssl req -config openssl.cnf -out out.csr -new -newkey rsa:2048 -nodes -keyout out.key
openssl x509 -signkey out.key -in out.csr -req -days 9999 -sha256 -extfile v3.ext -out out.crt
