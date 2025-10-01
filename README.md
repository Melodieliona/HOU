Das folgende Projekt setzt die pragmatische Variante der höheren Unifikation aus dem Paper Efficient Higher Order Unification von Petar Vukmirović, ohne Orakel, in Rust um.

Nach dem Herunterladen kann man den Algorithmus direkt über das Terminal mit dem Befehl `cargo run` starten.
Falls das Programm bereits zuvor verwendet wurde, könnte man nun die aus der vorherigen Sitzung selbst definierten Typen importieren.
Anschließend wird man gefragt, ob man neue Typen hinzufügen möchte. Die bereits verfügbaren Typen sind nat, bool und real.
Falls die vorherigen Typen importiert wurden, kann man nun auch die dazugehörigen Variablen aus der vorherigen Sitzung wiederherstellen oder löschen.

Im nächsten Schritt werden die Variablen definiert. Dabei wird zuerst der Name der neuen Variablen eingegeben. Großbuchstaben stehen hierbei für freie Variablen (auch bekannt als Metavariablen), während Kleinbuchstaben für gebundene Variablen oder Konstanten verwendet werden.

Danach deklariert man, ob es sich um eine freie Variable (Fvar) eine gebundene Variable (Bvar) oder um eine Konstante (Const) handelt.
Im letzten Schritt zur Deklaration der Variable fügt man den Typ ein, also ob es sich um einen Basis- oder Funktionstyp handelt (bspw. nat -> nat).

So entsteht eine Variable mit bspw.: x const nat

Dieser Vorgang wird solange ausgeführt, bis alle Variablen hinzugefügt wurden.

Es gibt die Möglichkeit hinzugefügte Variablen zu löschen, den Algorithmus abzubrechen oder mit weiter zum Erstellen des Constraints zu gelangen.

Ein Constraint besteht aus zwei Termen, die durch ?= voneinander getrennt sind. Lambdas kann man entweder als λ oder \l schreiben. Die einelnen Lambdawerte müssen durch Kommata getrennt werden und das Ende mit einem Punk gekennzeichnet bspw. λx,y,z. t (t steht hier für einen Term).

Das Programm prüft dann, ob die Typen der beiden Terme kompatibel sind. Ist das der Fall, wird man gefragt, ob die Anzahl der Bindungen begrenzt werden soll. Wenn nein ausgewählt wird, werden folgenden Standardgrenzen verwendet:

max_simple_proj: 4,
max_functional_proj: 4,
max_eliminations: 4,
max_imitations: 4,
max_identifications: 4,
max_total: 10,

Ansonsten kann man die Werte individuell anpassen oder leer lassen, falls man den Standardwert verwenden möchte.
Wird der Unifikationsprozess ausgeführt gibt er entweder die möglichen Lösungen zurück oder die Meldung "Es konnten keine Lösungen gefunden werden".
Die ausgegebenen Lösungen werden ebenfalls in solutions gespeichert.
