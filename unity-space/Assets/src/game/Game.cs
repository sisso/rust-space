using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading;
using core;
using Unity.Collections;
using UnityEngine;
using UnityEngine.Purchasing;
using utils;
using Debug = UnityEngine.Debug;

namespace game
{
    public class Game : MonoBehaviour
    {
        public domain.Domain domain;
        [Range(0.1f, 10.0f)] public float timeDilatation = 1f;
        [Range(1, 10)] public int iterations = 1;
        public double realTime = 0.0;
        public double gameTime = 0.0;
        public string[] init_arguments;
        public int lastRunTimeMls;
        private core.Core core;
        private Stopwatch stopWatch;
        
        void OnEnable()
        {
            if (this.core == null)
            {
                this.core = new core.Core("", this.domain);
                this.core.SetData(new Core.Request
                {
                    newGame = true,
                    arguments = this.init_arguments
                });
                this.stopWatch = new Stopwatch();
            }
        }

        void Destroy()
        {
            this.core.Dispose();
            this.core = null;
        }

        void FixedUpdate()
        {
            this.stopWatch.Reset();
            this.stopWatch.Start();

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

            this.stopWatch.Stop();
            TimeSpan ts = this.stopWatch.Elapsed;
            this.lastRunTimeMls = ts.Milliseconds;
            if (this.lastRunTimeMls > 1)
            {
                Debug.LogWarning($"native update time took {this.lastRunTimeMls}");
            }
        }
    }
}