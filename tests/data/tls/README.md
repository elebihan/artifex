# Artifex Test Data: TLS

Here are the commands to generate the keys and certificates required for running
an Artifex server and a client with a TLS connection.

## Root Certification Authority

Create a private key, then a self-signed certificate:

```sh
openssl genrsa -out root.key.pem 4096
cat<<'EOF'>root.cnf
basicConstraints = CA:TRUE
subjectKeyIdentifier = hash
authorityKeyIdentifier=keyid:always,issuer
keyUsage = critical,keyCertSign,cRLSign
EOF
openssl req -new -key root.key.pem -out root.csr.pem \
        -subj "/C=FR/ST=IDF/O=Example Organization/CN=Example Certification Authority"
openssl x509 -req -in root.csr.pem -out root.crt.pem \
        -signkey root.key.pem \
        -days 3650 \
        -extfile root.cnf
```

## Server

Create a private key, then a certificate, signed by the root CA:

```sh
openssl genrsa -out server.key.pem 4096
cat<<'EOF'>server.cnf
basicConstraints = CA:FALSE
nsCertType = server
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer:always
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = artifex-server
EOF
openssl req -new -key server.key.pem -out server.csr.pem \
        -subj "/C=FR/ST=IDF/O=Example Organization/CN=Example Server"
openssl x509 -req -in server.csr.pem -out server.crt.pem \
        -CA root.crt.pem \
        -CAkey root.key.pem \
        -days 3560 \
        -extfile server.cnf
```

Note the "alt_names" section in the extension, required for RustTLS to work.

Verify that the server certificate is signed by root CA:

```sh
openssl verify -CAfile root.crt.pem server.crt.pem
```

## Client

Create a private key, then a certificate, signed by the root CA:

```sh
openssl genrsa -out client.key.pem 4096
cat<<'EOF'>client.cnf
basicConstraints = CA:FALSE
nsCertType = client
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer:always
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth
EOF
openssl req -new -key client.key.pem -out client.csr.pem \
        -subj "/C=FR/ST=IDF/O=Example Organization/CN=Example Client"
openssl x509 -req -in client.csr.pem -out client.crt.pem \
        -CA root.crt.pem \
        -CAkey root.key.pem \
        -days 3560 \
        -extfile client.cnf
```

Verify that the client certificate is signed by root CA:

```sh
openssl verify -CAfile root.crt.pem client.crt.pem
```
