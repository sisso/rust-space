﻿using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using utils;

namespace game
{
    public class Game : MonoBehaviour
    {
        public domain.Domain domain;
        [Range(0.1f, 10.0f)]
        public float timeDilatation = 1f;
        [Range(1, 10)]
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
                // // get commands from domain to core
                // var requests = this.domain.TakeRequests();
                // this.core.Push(requests);
                // update time
                this.core.Update(delta);
                // get all data and send to domain
                this.core.GetData();
            }
        }
    }
}