#!/bin/sh
set -e
cd "$(dirname "$0")"

if [ ! -f ./target/release/pmrrepo ]; then
    cargo build --release
fi

./target/release/pmrrepo register https://models.physiomeproject.org/workspace/beeler_reuter_1977 'Beeler, Reuter, 1977'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/hodgkin_huxley_1952 'Hodgkin, Huxley, 1952'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/noble_1962 'Noble, 1962'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/clancy_rudy_2001 'Clancy, Rudy, 2001'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/demir_clark_giles_1999 'Demir, Clark, Giles, 1999'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/faber_rudy_2000 'Faber, Rudy, 2000'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/beard_2005 'Beard, 2005'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/saucerman_mcculloch_2004 'Saucerman, Mcculloch, 2004'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/reed_nijhout_sparks_ulrich_2004 'Reed, Nijhout, Sparks, Ulrich, 2004'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/baylor_hollingworth_chandler_2002 'Baylor, Hollingworth, Chandler, 2002'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/noble_noble_2001 'Noble, Noble, 2001'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/noble_varghese_kohl_noble_1998 'Noble, Varghese, Kohl, Noble, 1998'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/warren_tawhai_crampin_2009 'Warren, Tawhai, Crampin, 2009'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/bertram_previte_sherman_kinard_satin_2000 'Bertram, Previte, Sherman, Kinard, Satin, 2000'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/hunter_mcculloch_terkeurs_1998 'Hunter, Mcculloch, Terkeurs, 1998'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/hunter_mcnaughton_noble_1975 'Hunter, Mcnaughton, Noble, 1975'
./target/release/pmrrepo register https://models.physiomeproject.org/workspace/noble_difrancesco_denyer_1989 'Noble, Difrancesco, Denyer, 1989'

for id in $(seq 1 17); do
    ./target/release/pmrrepo sync ${id}
done
