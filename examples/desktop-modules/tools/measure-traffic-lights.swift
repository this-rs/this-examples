// Mesure la géométrie réelle des feux de fenêtre macOS.
//
// Pourquoi ce fichier est dans le dépôt : la taille des trois points, leur écart
// et leur marge suivent une NORME QUI CHANGE AVEC LA VERSION DE macOS. Recopier
// une valeur de mémoire donne un dessin faux de quelques pixels — c'est arrivé
// le 05/11/2026 : 12 px / écart 8 / marge 16 écrits à la main, alors que
// macOS 26.5.2 mesure 14 / 9 / 9.
//
// Rejouer sur la machine cible :   swift tools/measure-traffic-lights.swift
// Puis reporter dans ui/style.css (variables --win-dot, --win-gap, --win-inset,
// --win-bar).
//
// Ce que ça a rendu sur macOS 26.5.2 :
//   close x=9 y=9 14x14 · min x=32 · zoom x=55 · total 60 · barre 32 de haut
import AppKit

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let w = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 800, height: 600),
                 styleMask: [.titled, .closable, .miniaturizable, .resizable],
                 backing: .buffered, defer: false)
w.title = "mesure"

for (name, type) in [("close", NSWindow.ButtonType.closeButton),
                     ("min", .miniaturizeButton),
                     ("zoom", .zoomButton)] {
    if let f = w.standardWindowButton(type)?.frame {
        print("\(name): x=\(f.origin.x) y=\(f.origin.y) w=\(f.width) h=\(f.height)")
    }
}

print("hauteur de barre: \(w.frame.height - w.contentLayoutRect.height)")
if let close = w.standardWindowButton(.closeButton)?.frame,
   let mini = w.standardWindowButton(.miniaturizeButton)?.frame,
   let zoom = w.standardWindowButton(.zoomButton)?.frame {
    print("écart: \(mini.origin.x - (close.origin.x + close.width))")
    print("marge gauche: \(close.origin.x)")
    print("largeur des 3: \(zoom.origin.x + zoom.width - close.origin.x)")
}
