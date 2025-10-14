import importlib.util
import os
import sys

plugin_dir = os.path.dirname(__file__)
module_path = os.path.join(plugin_dir, "sitf_importer.py")

spec = importlib.util.spec_from_file_location("sitf_importer_module", module_path)
sitf_importer_module = importlib.util.module_from_spec(spec)
sys.modules["sitf_importer_module"] = sitf_importer_module
spec.loader.exec_module(sitf_importer_module)

SitfImporter = sitf_importer_module.SitfImporter
