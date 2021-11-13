using UnityEngine;
using System.Collections;
using System.Collections.Generic;

namespace utils
{
    public class DrawGizmosArrow : MonoBehaviour
    {
        public Color color = Color.blue;
        public Vector3 fromPos;
        public Vector3 toPos;
        public bool draw = true;

        void OnDrawGizmos()
        {
            if (draw)
            {
                Gizmos.color = color;
                Gizmos.DrawLine(this.fromPos, this.toPos);
            }
        }
    }
}