using UnityEngine;
using System.Collections;
using System.Collections.Generic;

namespace utils {
    public enum GizmoKind
    {
        Sphere, 
        Cube,
    } 

	public class DrawGizmos : MonoBehaviour {
		public Color color = Color.blue;
		public float radius = 1f;
		public Vector3 localPos;
		public bool draw = true;
        public GizmoKind kind;
		
		void OnDrawGizmos() {
			if (draw) {
                Gizmos.color = color;
                var pos = transform.TransformPoint(localPos);

                switch (this.kind) {
                    case GizmoKind.Sphere:
                        Gizmos.DrawWireSphere(pos, radius);
                        break;

                    case GizmoKind.Cube:
                        Gizmos.DrawWireCube(pos, Vector3.one * this.radius);
                        break;
                }
			}
		}
	}

}
