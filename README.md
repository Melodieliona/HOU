Um das Programm zu starten müssen Rust und Cargo installiert werden.
Danach kann man nach dem Herunterladen den Algorithmus direkt über das Terminal mit dem Befehl `cargo run` starten.

Beim Start können Typen und Variablen aus der vorherigen Sitzung importiert werden. Die Standarttypen sind Nat, Bool und Real.

Um Variablen anzulegen muss zuerst ein Name festgelegt werden. Großbuchstaben stehen hierbei für freie Variablen (Metavariablen), während Kleinbuchstaben für gebundene Variablen oder Konstanten verwendet werden.

Danach deklariert man, ob es sich um eine freie Variable (Fvar) eine gebundene Variable (Bvar) oder um eine Konstante (Const) handelt.
Im letzten Schritt wird Entweder ein Basis- oder Funktionstyp eingegeben, z.B.: nat oder nat->nat

So entsteht eine Variable mit bspw.: x const nat

Ein Constraint besteht aus zwei Termen, im Format term1 ?= term2. Lambdas können entweder als λ oder \l geschrieben werden. Einzelne Lambdawerte müssen durch Kommata getrennt werden und das Ende mit einem Punk gekennzeichnet bspw. λx,y,z. t (t steht hier für einen Term).

Das Programm prüft vor Unifikation, ob die beiden Terme typkompatibel sind.
Nach erfolgreicher Typprüfung fragt das Programm, ob Bindungsgrenzen gesetzt werden sollen. Wählt man „Nein“, gelten die Standardwerte:

max_simple_proj: 4,
max_functional_proj: 4,
max_eliminations: 4,
max_imitations: 4,
max_identifications: 4,
max_total: 10,

Alternativ können die Werte individuell eingegeben oder leer gelassen werden, um den jeweiligen Default zu übernehmen.
Wird der Unifikationsprozess ausgeführt gibt er entweder die möglichen Lösungen zurück oder die Meldung "Es konnten keine Lösungen gefunden werden".
Die ausgegebenen Lösungen werden ebenfalls in solutions gespeichert.
