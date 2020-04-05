using System.Collections;
using System.Collections.Generic;
using UnityEngine;

namespace game
{
    public class Game : MonoBehaviour
    {
        public domain.Domain domain;
        public float timeDilatation = 1f;
        public int iterations = 1;
        public double realTime = 0.0;
        public double gameTime = 0.0;
        private core.Core core;

        void OnEnable()
        {
            if (this.core == null)
            {
                this.core = new core.Core("", this.domain);
            }
        }

        void Destroy()
        {
            this.core.Dispose();
            this.core = null;
        }

        void FixedUpdate()
        {
            var delta = Time.fixedDeltaTime * timeDilatation;
            this.realTime += Time.fixedDeltaTime;
            for (int i = 0; i < iterations; i++)
            {
                this.gameTime += delta;
                this.core.Update(delta);
                this.core.GetData();

            }
        }
    }
}