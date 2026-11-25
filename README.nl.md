# efs-bill

## Waarom

Schat de maandkosten van **Amazon EFS** Standard en Infrequent Access (IA) per
GB-maand uit een lokale JSON-export, en toont een korte besparingsregel als de
export suggereert dat Standard-bytes naar IA kunnen.

Het leest alleen bestanden. Het gebruikt geen netwerk en heeft geen
AWS-inloggegevens.

De tests en de gemeten aantallen staan in `QUALITY.nl.md` en `QUALITY.md`.

## Installatie

Rust 1.85+. Vanuit een checkout:

```sh
cargo install --path .
```

## Snelstart

```sh
efs-bill examples/standard.json
efs-bill examples/ia.json --json
efs-bill examples/standard.json examples/ia.json --group-by env
```

## Aan de slag in 5 minuten

1. Installeer met `cargo install --path .` (zie Installatie).
2. Draai `efs-bill examples/standard.json`. Dat toont de Standard-kosten van
   één bestandssysteem.
3. Draai `efs-bill examples/ia.json --json` voor dezelfde schatting als JSON.
4. Wijs het naar je eigen export (een bestand of een map met `*.json`).
5. Lees Beperkingen en Prijzen en verantwoordelijkheid voordat je op een getal
   vertrouwt.

De voorbeelden worden alleen gelezen. Er wordt niets naar `examples/`
geschreven.

## Opties

```text
efs-bill [OPTIES] <PAD>...
```

- `<PAD>...`: JSON-exportbestanden, of een map met `*.json`-bestanden.
- `--prices BESTAND`: gebruik je eigen prijsbestand in plaats van de
  ingebouwde tarieven. Het formaat is dat van `prices/us-east-1.json`.
- `--group-by SLEUTEL`: groepeer rijen op deze `Tags`- of `Metadata`-sleutel.
- `--json`: toon JSON (`schema_version` 1) in plaats van de teksttabel.
- `--help`, `--version`.

## Voorbeeld

```sh
efs-bill examples/ia.json
```

Het rapport is een tabel met tabs: één rij per bestandssysteem en een
totaalrij, met Standard- en IA-bytes, Standard- en IA-kosten, het totaal en de
besparing als Standard-bytes naar IA gingen. Voor 1 GiB in IA geeft
`examples/ia.json` een IA-prijs van `0.016 USD`.

## Hoe het telt

- Invoer: een of meer JSON-bestanden, of een map met `*.json`, in de vorm
  `{ "FileSystems": [ { "FileSystemId", "Name"?, "StandardBytes"?, "IaBytes"?,
  "SizeInBytes"?, "LifecyclePolicies"?, "Tags"? } ] }` of één bestandssysteem.
  De velden volgen
  [FileSystemDescription](https://docs.aws.amazon.com/efs/latest/ug/API_FileSystemDescription.html)
  (geraadpleegd 2026-11-09).
- **Standard:** `StandardBytes / 1024³ × $0.30` van
  [Amazon EFS pricing](https://aws.amazon.com/efs/pricing/) (geraadpleegd
  2026-11-09). AWS meet opslag in binaire GB (`1 GB = 2³⁰` bytes).
- **IA:** `IaBytes / 1024³ × $0.016`. Dat is het us-east-1 Regional
  (Multi-AZ) **Elastic Throughput**-tarief op dezelfde pagina (prijslijst
  gepubliceerd 2026-11-04). Het IA-tarief voor legacy-throughput staat daar op
  $0.025; geef met `--prices` je eigen bestand mee voor dat tarief.
- `SizeInBytes.Value` en `ValueInIA` worden gebruikt als de expliciete
  Standard/IA-velden ontbreken.
- **IA-besparingstip:** als `LifecyclePolicies` (of `AccessPattern` /
  `SuggestIaBytes`) koude Standard-data suggereren, zet het tekstrapport een
  regel `# move … to IA → save …` met het verschil tussen Standard- en
  IA-tarief.
- `--group-by SLEUTEL` telt op per `Tags` of `Metadata`. Een groep toont één
  besparingsnotitie, gebouwd uit de besparing van de hele groep.
- **Geen AWS-velden:** `StandardBytes`, `IaBytes`, `StorageBytes`,
  `AccessPattern`, `SuggestIaBytes` en `LifecyclePolicies` zijn niet de velden
  van de AWS `FileSystemDescription`. Het is de invoervorm van dit hulpmiddel.
  Echt AWS heeft `SizeInBytes` en een aparte aanroep
  `describe-lifecycle-configuration`.
- **Lifecycle-tip:** alleen een beleid met een `TransitionToIA`-regel (terwijl
  Standard bytes heeft en IA er geen) adviseert de Standard-bytes te
  verplaatsen. Een beleid zonder die regel, zoals
  `TransitionToPrimaryStorageClass` / `AFTER_1_ACCESS`, geeft geen advies.
- **Grootte verplicht:** elk bestandssysteem heeft minstens één herkend
  grootteveld nodig (`StandardBytes`, `IaBytes`, `SizeInBytes.Value`/`ValueInIA`
  of `StorageBytes.Standard`/`IA`). Zonder, of bij een item dat geen JSON-object
  is, wordt de invoer geweigerd (exit 2) in plaats van als 0 geteld.
- Een map wordt alleen gelezen op bestanden met de kleine extensie `.json`. Een
  map zonder zo'n bestand is een fout, geen rekening van 0.
- **Tariefbereik:** elk tarief in een `--prices`-bestand is 0 of meer. Een
  negatief tarief wordt geweigerd (exit 2). Is het IA-tarief niet lager dan het
  Standard-tarief, dan wordt geen besparing beweerd. Een tarief of grootte
  waarbij het bedrag overloopt geeft exit 2.
- In de tekstentabel verschijnt een besturingsteken (regelwisseling, tab,
  escape) in naam, groep of valuta als `?`. `--json` escapet ze wel.

## Afsluitcodes

- `0`: het rapport is getoond (ook bij `--help` en `--version`).
- `2`: fout in invoer of gebruik, bijvoorbeeld een kapot JSON-bestand, een
  ontbrekend pad, een invoer zonder grootteveld, een negatief of overlopend
  tarief of een onbekende optie. De melding gaat naar stderr.

## Prijzen en verantwoordelijkheid

De ingebouwde tarieven zijn de us-east-1 Regional Elastic Throughput-tarieven
op de pagina [Amazon EFS pricing](https://aws.amazon.com/efs/pricing/),
geraadpleegd 2026-11-09. Ze staan met bron en datum in
`prices/us-east-1.json`. Dit project verzint geen prijzen. AWS kan prijzen op
elk moment wijzigen, en je rekening kan afwijken van deze schatting. Jij
controleert de tarieven tegen je eigen AWS-afspraken voordat je op een getal
handelt. Geef met `--prices` je eigen bestand mee.

## Beperkingen

- Alleen offline: geen AWS-netwerkclient in v0.1.
- De free tier wordt niet afgetrokken.
- Throughput, One Zone en Archive-klassen zijn niet gemodelleerd in v0.1.
  Het ingebouwde IA-tarief is Regional Elastic ($0.016), niet legacy IA
  ($0.025).
- Elk bestand is hoogstens 32 MiB.

## Onderhoud

De prijzen zijn een momentopname van 2026-11-09 en worden niet vanzelf
bijgewerkt. Verandert AWS een tarief, pas dan het prijsbestand aan (of geef
je eigen bestand mee met `--prices`) en noteer de nieuwe bron en datum in
`CHANGELOG.md`. Fouten en patches gaan via GitHub Issues, zie
`CONTRIBUTING.md`. Beveiligingsmeldingen gaan naar het adres in `SECURITY.md`.

## Ontwikkeling

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

De suite heeft 68 tests. Zie `QUALITY.nl.md`.

## Licentie

MIT OR Apache-2.0.

De licenties van de externe crates in het programma staan in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).
