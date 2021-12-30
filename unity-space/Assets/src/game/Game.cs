using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading;
using game.domain;
using space_data;
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
        public List<string> initialArguments;
        public int lastRunTimeMls;
        private Stopwatch stopWatch;
        private ffi_domain_2.SpaceGame game;

        void OnEnable()
        {
            if (this.game == null)
            {
                this.game = new ffi_domain_2.SpaceGame(this.initialArguments);
                this.stopWatch = new Stopwatch();
                
                this.CreateSectors();
            }
        }

        void Destroy()
        {
            if (this.game != null)
            {
                this.game.Dispose();
                this.game = null;
            }
        }

        void CreateSectors()
        {
            var sectors = this.game.GetSectors();
            foreach (var sector in sectors)
            {
                domain.AddSector(sector.Index(), new V2D(sector.Coords().Item1, sector.Coords().Item2));
            }
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
                // this.core.Update(delta);
                // // get all data and send to domain
                // this.core.GetData();
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