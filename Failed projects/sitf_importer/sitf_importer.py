from krita import *
import re

class SitfImporter(Extension):
    def __init__(self, parent):
        super().__init__(parent)

    def setup(self):
        pass

    def createActions(self, window):
        action = window.createAction("import_sitf", "Import SITF File", "file")
        action.triggered.connect(self.import_sitf)

    def parse_sitf(self, path):
        pixels = []
        with open(path, "r") as f:
            lines = f.readlines()

        reading = False
        for line in lines:
            line = line.strip()
            if line.startswith("@"):
                reading = True
                continue
            if not reading or not line:
                continue

            row = []
            entries = line.split(',')
            for entry in entries:
                match = re.search(r'([+-]?\d*\.?\d*)(!?[#%][\w/]+)', entry)
                if not match:
                    continue
                color_token = match.group(2)
                if color_token.startswith('!F'):
                    rgb = (255, 255, 255)
                elif color_token.startswith('!0'):
                    rgb = (0, 0, 0)
                elif color_token.startswith('!R'):
                    rgb = (255, 0, 0)
                elif color_token.startswith('!G'):
                    rgb = (0, 255, 0)
                elif color_token.startswith('!B'):
                    rgb = (0, 0, 255)
                else:
                    rgb = (128, 128, 128)
                row.append(rgb)
            pixels.append(row)
        return pixels

    def import_sitf(self):
        app = Krita.instance()
        filename = app.readSetting("", "LastVisitedDir", "")
        path = QFileDialog.getOpenFileName(None, "Open SITF File", filename, "SITF Files (*.sitf)")[0]
        if not path:
            return

        pixels = self.parse_sitf(path)
        if not pixels:
            QMessageBox.warning(None, "SITF Import", "Failed to parse SITF file.")
            return

        height = len(pixels)
        width = len(pixels[0])
        doc = app.createDocument(width, height, "SITF Image", "RGBA", "U8", "", 120.0)
        node = doc.rootNode()

        for y in range(height):
            for x in range(width):
                r, g, b = pixels[y][x]
                node.setPixel(x, y, r, g, b, 255)

        app.activeWindow().addView(doc)
        app.openDocument(doc)
        doc.refreshProjection()
        QMessageBox.information(None, "SITF Import", "SITF file imported successfully.")

# Register plugin
Krita.instance().addExtension(SitfImporter(Krita.instance()))
