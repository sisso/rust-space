using System.Collections;
using System.Collections.Generic;
using UnityEngine;

namespace game
{
    public class Game : MonoBehaviour
    {
        public domain.Domain domain;

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
            this.core.Update(Time.fixedDeltaTime);
            this.core.GetData();
        }
    }
}