# Security

## Reporting a vulnerability

Please report a security problem by e-mail to pep_ai@icloud.com. Do not open a
public issue for it. Say what you found, how to reproduce it, and which version
(`efs-bill --version`) you used.

## Scope

`efs-bill` reads local JSON files and prints a report. It makes no network
requests and holds no AWS credentials. Relevant reports are, for example,
a crash or a hang on crafted input, or an unsafe read of a path.

## Supported versions

Only the latest released version gets fixes.
