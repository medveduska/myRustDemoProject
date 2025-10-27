This is an app named Language Flashcards
It can import flashcards from csv file and then show them to user in randomized order. It can be either Word -> Translation or Translation -> Word
Main difference from other flahcard apps is that you can import up to 3 columns from CSV in UTF-8 format. As example could be Chinese language with character, pinyin and translation. e.g. CSV
阿姨,āyí,aunt
啊,a,ah
矮,ǎi,short

In this case I can learn Word -> -> pinyin -> Translation or Translation -> Word -> pinyin

This project contains code for an BE app written in rust lang, as well as frontend written in yew.
Cargo is used
Fe and Be are stored as separate modules.
