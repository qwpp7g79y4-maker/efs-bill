# Niet van toepassing

Eén regel per ontbrekend stuk: `stuk: reden`. Elk stuk bestaat echt niet in
deze repo.

build.rs: er is geen build-script nodig, want de crate heeft geen codegeneratie en geen native bibliotheek.
man: er is geen man-pagina; `efs-bill --help` en de README beschrijven de opties.
docs: er is geen aparte docs-map; de documentatie staat in README.md, README.nl.md en QUALITY.md.
samples: de voorbeeldbestanden staan al in examples/ en tests/fixtures/, een tweede map is overbodig.
scripts: er zijn geen hulpscripts; bouwen en testen gaan met cargo zelf.
.github/workflows/release.yml: er is geen releaseworkflow, want er wordt nog niets gepubliceerd en er komt geen tag.
