# Artifex Test Data: TLS

Here are the commands to generate the keys and certificates required for running
an Artifex server and a client with a TLS connection.

## File-based credentials
### Root Certification Authority

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

### Server

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
# DNS.1 = artifex-server
IP.1 = 127.0.0.1
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
Here an alternative IP is set allowing the client and the server running on the
same machine.

To allow the client to connect to the server using an alternative name (e.g
"artifex-server"), set the "alt_names" section to:

```
[alt_names]
DNS.1 = artifex-server
```

In that case, the CLI client should be run with `-n artifex-server` option or
its configuration file should contain 'server_alt_name = "artifex-server"' '"in
the "tls" section.


Verify that the server certificate is signed by root CA:

```sh
openssl verify -CAfile root.crt.pem server.crt.pem
```

### Client

Create password file for client private key:

```sh
echo "53cr3tP4ssw0rd" > client.key.password.txt
```

Create a private key, then a certificate, signed by the root CA:

```sh
openssl genrsa -aes256 -passout file:client.key.password.txt \
        -out client.key.pem 4096
cat<<'EOF'>client.cnf
basicConstraints = CA:FALSE
nsCertType = client
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer:always
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth
EOF
openssl req -new -key client.key.pem -out client.csr.pem \
        -passin file:client.key.password.txt \
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

## PKCS#11-based credentials

[PKCS11][PKCS11] is the standard to interact with cryptographic tokens.

[SoftHSM][SOFTHSM] is an implementation of a cryptographic store accessible
through a PKCS #11 interface. You can use it to explore PKCS #11 without having
a Hardware Security Module.

#### Installation

On Fedora GNU/Linux systems, [SoftHSM][SOFTHSM] can be installed as follows:

```sh
sudo dnf install -y softhsm p11-kit openssl
```

The user should configure the location of the virtual tokens:

```sh
mkdir -p ~/softhsm/tokens
mkdir -p ~/.config/softhsm2
echo "directories.tokendir = $HOME/softhsm/tokens" > ~/.config/softhsm2/softhsm2.conf
```

#### Token creation

To create a token named "Artifex Client Token 01" with "0000" as PIN for Security
Officer and "53cr3tP4ssw0rd" as PIN for the user, execute:

```sh
softhsm2-util --init-token --free \
              --label "Artifex Client Token 01" \
              --so-pin "0000" \
              --pin "53cr3tP4ssw0rd"
```

The output of the command should look like:

```
Slot 0 has a free/uninitialized token.
The token has been initialized and is reassigned to slot 1295994237
```

The [p11-kit][P11KIT] project provides tools to interact with PKCS#11 tokens.
Among them, `p11tool` can be used to check the presence of the newly created token:

```sh
p11tool --list-token-urls | grep SoftHSM
```

The output of the command should look like:

```
pkcs11:model=SoftHSM%20v2;manufacturer=SoftHSM%20project;serial=54dc4636118a5a86;token=Artifex%20Client%20Token%2001
```

#### Import client private key

To import the RSA private key for the client named
``tests/data/tls/client.key.pem`` in the token as "Artifex Client Key 01",
execute:

```sh
p11tool --login --set-pin '53cr3tP4ssw0rd' \
        --load-privkey=tests/data/tls/client.key.pem \
        --label 'Artifex Client Key 01' \
        --mark-sign \
        --mark-decrypt \
        --write \
        pkcs11:token=Artifex%20Client%20Token%2001
```

To check the key has been properly imported, execute:

```sh
p11tool --login --set-pin '53cr3tP4ssw0rd' \
        --list-privkeys \
        pkcs11:token=Artifex%20Client%20Token%2001
```

The output of the command should look like:

```
Object 0:
        URL: pkcs11:model=SoftHSM%20v2;manufacturer=SoftHSM%20project;serial=146839644d3f4d7d;token=Artifex%20Client%20Token%2001;id=%56%95%F0%8E%EA%07%51%FD%6D%ED%81%E6%3D%93%D6%82%C3%66%30%77;object=Artifex%20Client%20Key%2001;type=private
        Type: Private key (RSA-4096)
        Label: Artifex Client Key 01
        Flags: CKA_WRAP/UNWRAP; CKA_PRIVATE; CKA_SENSITIVE;
        ID: 56:95:f0:8e:ea:07:51:fd:6d:ed:81:e6:3d:93:d6:82:c3:66:30:77
```

To test signature, execute:

```sh
p11tool --login --set-pin '53cr3tP4ssw0rd' \
        --test-sign \
        --hash=SHA512 \
        'pkcs11:token=Artifex%20Client%20Token%2001;object=Artifex%20Client%20Key%2001'
```

#### Import client certificate

To import the certificate for the client named ``tests/data/tls/client.crt.pem``
in the token as "Artifex Client Certificate 01", execute:

```sh
p11tool --login --set-pin '53cr3tP4ssw0rd' \
        --load-certificate=tests/data/tls/client.crt.pem \
        --label 'Artifex Client Certificate 01' \
        --write \
        pkcs11:token=Artifex%20Client%20Token%2001
```

The output must look like this:

```
note: will reuse ID 5695f08eea0751fd6ded81e63d93d682c3663077 from corresponding private key
```

To check the certificate has been properly imported, execute:

```sh
p11tool --login --set-pin '53cr3tP4ssw0rd' \
        --list-certs \
        pkcs11:token=Artifex%20Client%20Token%2001
```

The output of the command should look like:

```
Object 0:
        URL: pkcs11:model=SoftHSM%20v2;manufacturer=SoftHSM%20project;serial=146839644d3f4d7d;token=Artifex%20Client%20Token%2001;id=%56%95%F0%8E%EA%07%51%FD%6D%ED%81%E6%3D%93%D6%82%C3%66%30%77;object=Artifex%20Client%20Certificate%2001;type=cert
        Type: X.509 Certificate (RSA-4096)
        Expires: Mon Feb 19 10:29:16 2035
        Label: Artifex Client Certificate 01
        ID: 56:95:f0:8e:ea:07:51:fd:6d:ed:81:e6:3d:93:d6:82:c3:66:30:77
```

It is possible to read the certificate by executing:

```sh
p11tool --export \
        "pkcs11:token=Artifex%20Client%20Token%2001;object=Artifex%20Client%20Certificate%2001"
```

#### Import root CA certificate

The same procedure as above can be used to import the root CA certificate.

```sh
p11tool --login --set-pin '53cr3tP4ssw0rd' \
        --load-certificate=tests/data/tls/root.crt.pem \
        --label 'Artifex Root CA Certificate 01' \
        --mark-ca \
        --write \
        pkcs11:token=Artifex%20Client%20Token%2001
```

To list the certificates, execute:

```sh
p11tool --login --set-pin '53cr3tP4ssw0rd' \
        --list-certs \
        pkcs11:token=Artifex%20Client%20Token%2001
```

The output should now be:

```
Object 0:
        URL: pkcs11:model=SoftHSM%20v2;manufacturer=SoftHSM%20project;serial=146839644d3f4d7d;token=Artifex%20Client%20Token%2001;id=%56%95%F0%8E%EA%07%51%FD%6D%ED%81%E6%3D%93%D6%82%C3%66%30%77;object=Artifex%20Client%20Certificate%2001;type=cert
        Type: X.509 Certificate (RSA-4096)
        Expires: Mon Feb 19 10:29:16 2035
        Label: Artifex Client Certificate 01
        ID: 56:95:f0:8e:ea:07:51:fd:6d:ed:81:e6:3d:93:d6:82:c3:66:30:77

Object 1:
        URL: pkcs11:model=SoftHSM%20v2;manufacturer=SoftHSM%20project;serial=146839644d3f4d7d;token=Artifex%20Client%20Token%2001;id=%61%6F%DE%6A%34%72%65%02%90%67%30%15%DE%05%5E%B8%E5%5B%BF%70;object=Artifex%20Root%20CA%20Certificate%2001;type=cert
        Type: X.509 Certificate (RSA-4096)
        Expires: Thu May  3 16:30:38 2035
        Label: Artifex Root CA Certificate 01
        ID: 61:6f:de:6a:34:72:65:02:90:67:30:15:de:05:5e:b8:e5:5b:bf:70
```

[SOFTHSM]: https://www.opendnssec.org/softhsm/
[P11KIT]: https://p11-glue.github.io/p11-glue/p11-kit.html
[PKCS11]: https://en.wikipedia.org/wiki/PKCS_11
