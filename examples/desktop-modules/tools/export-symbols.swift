import AppKit

// Exporte chaque symbole système en PNG (3x) et imprime son base64 : ce sont
// les dessins d'Apple, pas une reconstitution.
let app = NSApplication.shared
app.setActivationPolicy(.accessory)

let targets: [(String, String)] = [
    ("close", "xmark"),
    ("minimize", "minus"),
    ("zoom", "arrow.up.left.and.arrow.down.right"),
]

for (label, name) in targets {
    guard let base = NSImage(systemSymbolName: name, accessibilityDescription: nil) else {
        print("\(label): ABSENT"); continue
    }
    let side = 42.0                       // 14 px de point × 3
    let configured = base.withSymbolConfiguration(
        NSImage.SymbolConfiguration(pointSize: 14 * 3 * 0.62, weight: .bold))!
    let image = NSImage(size: NSSize(width: side, height: side))
    image.lockFocus()
    NSColor.clear.setFill()
    NSRect(x: 0, y: 0, width: side, height: side).fill()
    let size = configured.size
    configured.draw(in: NSRect(x: (side - size.width) / 2, y: (side - size.height) / 2,
                               width: size.width, height: size.height))
    image.unlockFocus()

    let rep = NSBitmapImageRep(data: image.tiffRepresentation!)!
    let png = rep.representation(using: .png, properties: [:])!
    print("\(label)\t\(png.base64EncodedString())")
}
