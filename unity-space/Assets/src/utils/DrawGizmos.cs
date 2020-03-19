using UnityEngine;
using System.Collections;
using System.Collections.Generic;

namespace utils {
	 
	public class DrawGizmos : MonoBehaviour {
		public Color color = Color.blue;
		public float radius = 1f;
		public Vector3 localPos;
		public bool draw = true;
		
		void OnDrawGizmos() {
			if (draw) {
				Gizmos.color = color;
				var pos = transform.TransformPoint(localPos);
				Gizmos.DrawWireSphere(pos, radius);
			}
		}
	}

}
