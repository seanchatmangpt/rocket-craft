import sys
sys.path.insert(0, "/Users/sac/rocket-craft/scripts")
import verify_metric_morphology as vm
import pprint

parts = vm.measure_all()
pprint.pprint(parts['SM_Blade_Left'])
print("VERT:", vm.VERT)
